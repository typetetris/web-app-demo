use std::{ops::ControlFlow, pin::pin};

use actix_web::{
    App, HttpRequest, HttpResponse, Responder,
    dev::ServiceFactory,
    error, get,
    http::{StatusCode, header::ContentType},
    web::{self, Bytes, PathConfig},
};
use actix_ws::{AggregatedMessage, CloseReason, Closed, MessageStream, ProtocolError, Session};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio_stream::wrappers::BroadcastStream;
use tracing::instrument;
use tracing_actix_web::TracingLogger;
use uuid::Uuid;

use crate::chat::{
    ChatServer, ChatServerErrors,
    models::{ChatId, ChatMessage, ChatTimestamp, DisplayName, EventId, Message, UserId},
};

#[derive(Debug, Error)]
enum EndpointErrors {
    #[error("Server Error")]
    InternalServerError,

    #[error("Chat Not Found {0}")]
    ChatNotFound(ChatId),
}

impl error::ResponseError for EndpointErrors {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::plaintext())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            EndpointErrors::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            EndpointErrors::ChatNotFound(_) => StatusCode::NOT_FOUND,
        }
    }
}

impl From<ChatServerErrors> for EndpointErrors {
    fn from(value: ChatServerErrors) -> Self {
        match value {
            ChatServerErrors::LockPoisoned { backtrace, message } => {
                tracing::error!(message, ?backtrace, "lock poisoned error");
                EndpointErrors::InternalServerError
            }
            ChatServerErrors::ChatNotFound { chat_id } => EndpointErrors::ChatNotFound(chat_id),
        }
    }
}

#[get("/history/{chat_id}")]
#[instrument(skip(app_state))]
pub async fn get_chat_history(
    path_parameter: web::Path<Uuid>,
    app_state: web::Data<ChatServer>,
) -> Result<impl Responder, EndpointErrors> {
    let chat_id = ChatId::from_uuid(path_parameter.into_inner());
    let history = app_state.get_chat_history(chat_id)?;
    Ok(web::Json(history))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IncomingChatMessage {
    pub display_name: DisplayName,
    pub message: Message,
}

#[derive(Debug)]
enum IncomingStreamEventSuccess {
    ChatMessage(IncomingChatMessage),
    Ping(Bytes),
    Close(Option<CloseReason>),
}

#[derive(Debug)]
enum IncomingStreamEventError {
    ProtocolError(ProtocolError),
    ParseError(serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum Outgoing {
    ChatMessage { msg: ChatMessage },
    Error { msg: String },
}

type IncomingStreamEvent = Result<IncomingStreamEventSuccess, IncomingStreamEventError>;

#[instrument]
async fn preprocess_incoming_stream_event(
    msg: Result<AggregatedMessage, ProtocolError>,
) -> Option<IncomingStreamEvent> {
    match msg {
        Ok(inner_message) => match inner_message {
            AggregatedMessage::Text(byte_string) => Some(
                serde_json::from_slice(byte_string.as_ref())
                    .map(|v| IncomingStreamEventSuccess::ChatMessage(v))
                    .map_err(|err| IncomingStreamEventError::ParseError(err)),
            ),
            AggregatedMessage::Binary(_) => {
                tracing::warn!("unexpected binary message received");
                None
            }
            AggregatedMessage::Ping(bytes) => Some(Ok(IncomingStreamEventSuccess::Ping(bytes))),
            AggregatedMessage::Pong(_) => {
                tracing::warn!("unexpected pong message received");
                None
            }
            AggregatedMessage::Close(close_reason) => {
                Some(Ok(IncomingStreamEventSuccess::Close(close_reason)))
            }
        },
        Err(err) => Some(Err(IncomingStreamEventError::ProtocolError(err))),
    }
}

#[derive(Debug, Error)]
enum SendMessageError {
    #[error("failed to serialize outgoing message: {0}")]
    SerializationFailure(#[from] serde_json::Error),

    #[error("failed to send message, websocket closed: {0}")]
    Closed(#[from] Closed),
}

async fn send_message(session: &mut Session, message: Outgoing) -> Result<(), SendMessageError> {
    let serialized_message = serde_json::to_string(&message)?;
    session.text(serialized_message).await?;
    Ok(())
}

#[instrument(skip(chat_server, session))]
async fn handle_incoming_stream_event(
    chat_id: ChatId,
    user_id: UserId,
    stream_event: Option<IncomingStreamEvent>,
    chat_server: &ChatServer,
    session: &mut Session,
) -> ControlFlow<(), ()> {
    match stream_event {
        Some(Ok(msg)) => match msg {
            IncomingStreamEventSuccess::ChatMessage(incoming_chat_message) => {
                if let Err(err) = chat_server.send_message(ChatMessage {
                    event_id: EventId::random(),
                    timestamp: ChatTimestamp::now(),
                    chat_id,
                    user_id,
                    display_name: incoming_chat_message.display_name,
                    message: incoming_chat_message.message,
                }) {
                    tracing::error!(?err, "error sending message to chat");
                    return ControlFlow::Break(());
                }
            }
            IncomingStreamEventSuccess::Ping(bytes) => {
                if let Err(err) = session.pong(&bytes).await {
                    tracing::error!(?err, "error sending pong");
                    return ControlFlow::Break(());
                }
            }
            IncomingStreamEventSuccess::Close(close_reason) => {
                tracing::info!(?close_reason, "connection closed");
                return ControlFlow::Break(());
            }
        },
        Some(Err(err)) => match err {
            IncomingStreamEventError::ProtocolError(err) => {
                tracing::error!(%err, "protocol error");
                return ControlFlow::Break(());
            }
            IncomingStreamEventError::ParseError(error) => {
                let msg = format!("couldn't parse incoming message: {error}");
                tracing::warn!("{}", msg);
                if let Err(err) = send_message(session, Outgoing::Error { msg }).await {
                    tracing::error!(?err, "error sending message to websocket");
                    return ControlFlow::Break(());
                }
            }
        },
        None => {
            tracing::info!("websocket connection closed");
            return ControlFlow::Break(());
        }
    }
    ControlFlow::Continue(())
}

#[instrument(skip(chat_server, session, stream))]
pub async fn handle_websocket_connection(
    chat_id: ChatId,
    user_id: UserId,
    chat_server: web::Data<ChatServer>,
    mut session: Session,
    stream: MessageStream,
    mut broadcast: BroadcastStream<ChatMessage>,
) {
    let stream = stream
        .aggregate_continuations()
        .max_continuation_size(2usize.pow(22))
        .filter_map(preprocess_incoming_stream_event);

    let mut pinned_stream = pin!(stream);
    loop {
        tokio::select! {
            incoming_stream_event = pinned_stream.next() => {
                if let ControlFlow::Break(()) =
                    handle_incoming_stream_event(chat_id, user_id, incoming_stream_event, &chat_server, &mut session)
                        .await
                {
                    break;
                }
            },
            outgoing_message = broadcast.next() => {
                match outgoing_message {
                    Some(Ok(message)) => {
                        if let Err(err) =
                            send_message(&mut session, Outgoing::ChatMessage { msg: message }).await
                        {
                            tracing::error!(?err, "failed to send message to websocket");
                            break;
                        }
                    }
                    Some(Err(err)) => {
                        tracing::error!(?err, "receiving chat messages from chat server");
                        break;
                    },
                    None => {
                        tracing::error!("message stream from chat server closed");
                        break;
                    }
                }
            }
        }
    }
    tracing::info!("leaving chat");
    chat_server.part_chat(chat_id);
}

#[get("/chat/{chat_id}/{user_id}")]
#[instrument(skip(app_state, stream))]
pub async fn connect_to_chat(
    app_state: web::Data<ChatServer>,
    path_parameters: web::Path<(Uuid, Uuid)>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let (chat_uuid, user_uuid) = path_parameters.into_inner();
    let chat_id = ChatId::from_uuid(chat_uuid);
    let user_id = UserId::from_uuid(user_uuid);

    let (res, session, stream) = actix_ws::handle(&req, stream)?;
    let chat_messages_receiver = app_state.join_chat(chat_id);

    actix_web::rt::spawn(handle_websocket_connection(
        chat_id,
        user_id,
        app_state,
        session,
        stream,
        chat_messages_receiver,
    ));

    Ok(res)
}

pub fn setup_app(
    chat_server: web::Data<ChatServer>,
) -> App<
    impl ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse<
            tracing_actix_web::StreamSpan<actix_web::body::BoxBody>,
        >,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    return App::new()
        .wrap(TracingLogger::default())
        .app_data(chat_server)
        .app_data(PathConfig::default().error_handler(|err, _| err.into()))
        .service(get_chat_history)
        .service(connect_to_chat);
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use actix_http::ws::{self, Frame};
    use actix_test::TestServer;
    use actix_web::{
        http::{StatusCode, header::Accept},
        test, web,
    };
    use futures::{SinkExt, StreamExt};
    use tokio::time::error::Elapsed;

    use crate::{
        chat::{
            models::{ChatId, ChatMessage, DisplayName, Message, UserId}, ChatServer
        },
        services::{setup_app, IncomingChatMessage, Outgoing},
    };

    #[test_log::test(tokio::test)]
    async fn requesting_a_history_for_an_unknown_chat_yields_404() {
        let chat_server = web::Data::new(ChatServer::new());
        let app = test::init_service(setup_app(chat_server)).await;
        let req = test::TestRequest::get()
            .uri("/history/f48d88c2-efe7-462f-97ca-3b6350e1a1a4")
            .insert_header(Accept::json())
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test_log::test(tokio::test)]
    async fn requesting_a_history_for_an_unparsable_chat_id_yields_400() {
        let chat_server = web::Data::new(ChatServer::new());
        let app = test::init_service(setup_app(chat_server)).await;
        let req = test::TestRequest::get()
            .uri("/history/slartibartfass")
            .insert_header(Accept::json())
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    fn create_testserver() -> TestServer {
        let chat_server = web::Data::new(ChatServer::new());
        actix_test::start(move || setup_app(chat_server.clone()))
    }

    #[test_log::test(actix_web::test)]
    async fn connecting_the_websocket_for_an_unknown_chat_succeeds_and_creates_the_chat() {
        let mut app = create_testserver();

        let chat_id = ChatId::random();
        let user_id = UserId::random();

        let _framed = app
            .ws_at(&format!("/chat/{chat_id}/{user_id}"))
            .await
            .unwrap();
        let mut history_response = app
            .get(&format!("/history/{chat_id}"))
            .insert_header(Accept::json())
            .send()
            .await
            .unwrap();
        assert_eq!(history_response.status(), StatusCode::OK);
        let history: Vec<ChatMessage> = history_response.json().await.unwrap();
        assert_eq!(history, vec![]);
    }

    fn chat_message_as_ws_text(display_name: String, message: String) -> ws::Message {
        ws::Message::Text(
            serde_json::to_string(&IncomingChatMessage {
                display_name: DisplayName::new(display_name),
                message: Message::new(message),
            })
            .unwrap()
            .into(),
        )
    }

    #[test_log::test(actix_web::test)]
    async fn messages_sent_to_a_chat_are_in_the_history() {
        let mut app = create_testserver();

        let chat_id = ChatId::random();
        let user_id = UserId::random();

        let mut framed = app
            .ws_at(&format!("/chat/{chat_id}/{user_id}"))
            .await
            .unwrap();

        framed
            .send(chat_message_as_ws_text("Hugo".to_string(), "Nachricht 1".to_string()))
            .await
            .unwrap();

        framed
            .send(chat_message_as_ws_text("Hugo".to_string(), "Nachricht 2".to_string()))
            .await
            .unwrap();

        // reading the messages back, so we can be sure, they should have been added to the
        // history now.

        tokio::time::timeout(Duration::from_millis(100), framed.next())
            .await
            .context("sent messages not echoed back")
            .unwrap()
            .unwrap()
            .unwrap();
        tokio::time::timeout(Duration::from_millis(100), framed.next())
            .await
            .context("sent messages not echoed backe")
            .unwrap()
            .unwrap()
            .unwrap();

        let mut history_response = app
            .get(&format!("/history/{chat_id}"))
            .insert_header(Accept::json())
            .send()
            .await
            .unwrap();

        assert_eq!(history_response.status(), StatusCode::OK);
        let history: Vec<ChatMessage> = history_response.json().await.unwrap();

        let messages: Vec<_> = history.into_iter().map(|cm| cm.message).collect();
        pretty_assertions::assert_eq!(messages, vec![
            Message::new("Nachricht 1".to_string()),
            Message::new("Nachricht 2".to_string())
        ])
    }

    #[test_log::test(actix_web::test)]
    async fn a_message_sent_to_a_chat_will_be_echoed_back() {
        let mut app = create_testserver();

        let chat_id = ChatId::random();
        let user_id = UserId::random();

        let mut framed = app
            .ws_at(&format!("/chat/{chat_id}/{user_id}"))
            .await
            .unwrap();

        framed
            .send(chat_message_as_ws_text("Hugo".to_string(), "Nachricht 1".to_string()))
            .await
            .unwrap();

        let message = framed.next().await.unwrap().unwrap();
        let Frame::Text(bytes) = message else {
            panic!("Didn't receive a text frame");
        };
        let message: Outgoing = serde_json::from_slice(&bytes).unwrap();
        let Outgoing::ChatMessage { msg } = message else {
            panic!("Didn't receive a chat message");
        };
        assert_eq!(msg.message, Message::new("Nachricht 1".to_string()));
    }

    #[test_log::test(actix_web::test)]
    async fn a_message_sent_to_a_chat_will_not_be_received_in_a_different_chat() {
        let mut app = create_testserver();

        let chat_id_1: ChatId = ChatId::random();
        let chat_id_2: ChatId = ChatId::random();

        let user_id: UserId = UserId::random();

        let mut ws_chat1 = app
            .ws_at(&format!("/chat/{chat_id_1}/{user_id}"))
            .await
            .unwrap();

        let mut ws_chat2 = app
            .ws_at(&format!("/chat/{chat_id_2}/{user_id}"))
            .await
            .unwrap();

        ws_chat1
            .send(chat_message_as_ws_text("Hugo".to_string(), "Chat 1".to_string()))
            .await
            .unwrap();

        assert!(tokio::time::timeout(Duration::from_millis(100), ws_chat2.next()).await.is_err(), "didn't expect to receive a message in second chat");
    }
}
