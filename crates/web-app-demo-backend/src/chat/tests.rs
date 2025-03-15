use anyhow::Context;
use futures::{FutureExt as _, future::try_join_all};
use tokio_stream::StreamExt;

use super::*;
use super::models::*;

macro_rules! expect_no_message_available {
    ($receiver:expr, $mesg:expr) => {
        assert!($receiver.try_next().now_or_never().is_none(), $mesg);
    };
}

macro_rules! expect_message {
    ($receiver:expr, $mesg:expr) => {{
        let result = $receiver.try_next().now_or_never();
        if let Some(channel_event) = result {
            match channel_event {
                Ok(Some(message)) => message,
                Ok(None) => {
                    panic!("{}: stream ended unexpectedly", $mesg)
                }
                Err(err) => {
                    panic!("{}: error receiving message {err:#?}", $mesg);
                }
            }
        } else {
            panic!("{}: no message received", $mesg);
        }
    }};
}

fn test_message(chat_id: ChatId, user_id: UserId, event_id: EventId) -> ChatMessage {
    ChatMessage {
        event_id,
        timestamp: ChatTimestamp::epoch(),
        chat_id,
        user_id,
        display_name: DisplayName::new(format!("{}", user_id)),
        message: Message::new(format!("{}", user_id)),
    }
}

#[test]
fn fetching_the_history_of_an_unknown_chat_should_fail_with_the_correct_chat_id_in_the_error_message()
 {
    let sut = ChatServer::new();
    let chat_id = ChatId::random();

    let result = sut.get_chat_history(chat_id);
    // Hopefully assert_matches is stabilized soon.
    assert!(
        {
            match result {
                Err(ChatServerErrors::ChatNotFound {
                    chat_id: chat_id_in_err,
                }) => chat_id_in_err == chat_id,
                _ => false,
            }
        },
        "fetching the history of an unknown chat should fail with the correct chat id in the error: {result:#?}"
    );
}

#[test]
fn sending_a_message_to_an_unknown_chat_should_succeed_and_create_the_chat()
-> anyhow::Result<()> {
    let sut = ChatServer::new();
    let chat_id = ChatId::random();
    let user_id = UserId::random();
    let event_id = EventId::random();
    let message = test_message(chat_id, user_id, event_id);

    sut.send_message(message)
        .context("sending a message to an unknown chat should succeed")?;
    sut.get_chat_history(chat_id)
        .context("sending a message should create the chat if necessary")?;

    Ok(())
}

#[test]
fn joining_an_unknown_chat_should_succeed_and_create_the_chat() -> anyhow::Result<()> {
    let sut = ChatServer::new();
    let chat_id = ChatId::random();
    let _stream = sut.join_chat(chat_id);
    sut.get_chat_history(chat_id)
        .context("joining an unknown chat should create the chat if necessary")?;
    Ok(())
}

#[test]
fn user_should_receive_messages_for_the_chat_they_joined() -> anyhow::Result<()> {
    let sut = ChatServer::new();
    let chat_id = ChatId::random();
    let user_id = UserId::random();
    let event_id = EventId::random();
    let message = test_message(chat_id, user_id, event_id);

    let mut receiver = sut.join_chat(chat_id);
    sut.send_message(message)
        .context("sending a message should succeed")?;

    let received_message = expect_message!(receiver, "receiving the sent message");
    assert_eq!(
        received_message.event_id, event_id,
        "wrong event id received"
    );

    expect_no_message_available!(
        receiver,
        "no message should be available, as only one was sent"
    );

    Ok(())
}

fn send_test_message(
    sut: &ChatServer,
    chat: models::ChatId,
    user: models::UserId,
    event_id: models::EventId,
) -> anyhow::Result<()> {
    let message = test_message(chat, user, event_id);
    sut.send_message(message)?;
    Ok(())
}

#[tokio::test]
async fn concurrently_sending_messages_to_multiple_different_chats_the_correct_chats_should_receive_the_messages_and_the_histories_should_be_correct()
-> anyhow::Result<()> {
    let sut = Arc::new(ChatServer::new());

    let user1 = models::UserId::random();
    let user2 = models::UserId::random();

    let chat1 = models::ChatId::random();
    let chat2 = models::ChatId::random();

    let mut receiver_for_chat1 = {
        let sut = sut.clone();
        sut.join_chat(chat1)
    };
    let mut receiver_for_chat2 = {
        let sut = sut.clone();
        sut.join_chat(chat2)
    };

    let event1_chat1 = models::EventId::random();
    let event2_chat1 = models::EventId::random();

    let event1_chat2 = models::EventId::random();
    let event2_chat2 = models::EventId::random();

    let barrier = Arc::new(tokio::sync::Barrier::new(4));

    try_join_all(
        [
            (user1, chat1, event1_chat1),
            (user2, chat2, event1_chat2),
            (user1, chat2, event2_chat2),
            (user2, chat1, event2_chat1),
        ]
        .into_iter()
        .map(|(user, chat, event)| {
            let sut = sut.clone();
            let barrier = barrier.clone();
            tokio::spawn(async move {
                barrier.wait().await;
                send_test_message(&sut, chat, user, event)
            })
        }),
    )
    .await
    .context("joining a sending job failed")?
    .into_iter()
    .for_each(|job_result| {
        if let Err(err) = job_result {
            panic!("sending message in one of the jobs failed: {err}");
        }
    });

    let message1_for_chat1 = expect_message!(receiver_for_chat1, chat1);
    assert!(
        message1_for_chat1.event_id == event1_chat1
            || message1_for_chat1.event_id == event2_chat1,
        "wrong event id received on chat1 {}",
        message1_for_chat1
    );
    let message2_for_chat1 = expect_message!(receiver_for_chat1, chat1);
    assert!(
        message2_for_chat1.event_id == event1_chat1
            || message2_for_chat1.event_id == event2_chat1,
        "wrong event id received on chat1 {}",
        message2_for_chat1
    );
    let message1_for_chat2 = expect_message!(receiver_for_chat2, chat2);
    assert!(
        message1_for_chat2.event_id == event1_chat2
            || message1_for_chat2.event_id == event2_chat2,
        "wrong event id received on chat2 {}",
        message1_for_chat2
    );
    let message2_for_chat2 = expect_message!(receiver_for_chat2, chat2);
    assert!(
        message2_for_chat2.event_id == event1_chat2
            || message2_for_chat2.event_id == event2_chat2,
        "wrong event id received on chat2 {}",
        message2_for_chat2
    );

    expect_no_message_available!(
        receiver_for_chat1,
        "more than 2 messages available in chat1"
    );
    expect_no_message_available!(
        receiver_for_chat2,
        "more than 2 messages available in chat2"
    );

    let chat1_history = sut
        .get_chat_history(chat1)
        .context("history of chat1 not available")?;
    assert_eq!(
        chat1_history.len(),
        2,
        "History of chat1 has not exactly 2 chat messages"
    );
    chat1_history.iter().for_each(|chat_message| {
        assert!(
            chat_message.event_id == event1_chat1 || chat_message.event_id == event2_chat1,
            "wrong event id in history of chat1 {}",
            chat_message
        );
    });

    let chat2_history = sut
        .get_chat_history(chat2)
        .context("history of chat2 not available")?;
    assert_eq!(
        chat2_history.len(),
        2,
        "History of chat2 has not exactly 2 chat messages"
    );
    chat2_history.iter().for_each(|chat_message| {
        assert!(
            chat_message.event_id == event1_chat2 || chat_message.event_id == event2_chat2,
            "wrong event id in history of chat2 {}",
            chat_message
        );
    });

    Ok(())
}

#[test]
fn removing_the_last_receiver_should_cleanup_the_broadcast_map() {
    let sut = ChatServer::new();
    let chat_id = models::ChatId::random();
    
    let receiver1 = sut.join_chat(chat_id);
    
    let receiver2 = sut.join_chat(chat_id);
    assert_eq!(sut.broadcasts.len(), 1, "having two receivers the chat should have a broadcast map entry");
    drop(receiver2);
    // dropped receiver2, now parting the chat
    sut.part_chat(chat_id);

    assert_eq!(sut.broadcasts.len(), 1, "having one receiver the chat should have a broadcast map entry");
    drop(receiver1);
    // dropped recever1, now parting the chat
    sut.part_chat(chat_id);
    assert_eq!(sut.broadcasts.len(), 0, "having now receiver any more and having called `part_chat` triggering the cleanup, there should be no broadcast map entry any more");
}
