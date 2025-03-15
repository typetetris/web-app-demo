use std::{backtrace::Backtrace, fmt::Display, sync::Arc};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio_stream::wrappers::BroadcastStream;

use crate::util::wrappedbacktrace::WrappedBacktrace;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UserId(uuid::Uuid);

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl UserId {
    pub fn random() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ChatId(uuid::Uuid);

impl Display for ChatId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ChatId {
    pub fn random() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EventId(uuid::Uuid);

impl Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl EventId {
    pub fn random() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DisplayName(String);

impl Display for DisplayName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DisplayName {
    pub fn new(display_name: String) -> Self {
        Self(display_name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Message(String);

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Message {
    pub fn new(message: String) -> Self {
        Self(message)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ChatTimestamp(DateTime<Utc>);

impl Display for ChatTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        )
    }
}

impl ChatTimestamp {
    pub fn now() -> Self {
        ChatTimestamp(chrono::Utc::now())
    }

    pub fn epoch() -> Self {
        ChatTimestamp(chrono::DateTime::UNIX_EPOCH)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ChatMessage {
    pub event_id: EventId,
    pub timestamp: ChatTimestamp,
    pub chat_id: ChatId,
    pub user_id: UserId,
    pub display_name: DisplayName,
    pub message: Message,
}

impl Display for ChatMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ts = &self.timestamp;
        let ch = self.chat_id;
        let ev = self.event_id;
        let us = self.user_id;
        let dn = &self.display_name;
        let me = &self.message;
        write!(
            f,
            "{ts}: Chat {ch} Event {ev} User {us} DisplayName {dn}: {me}"
        )
    }
}

pub struct ChatServer {
    // We intentionally use a std::sync::Mutex, as we never expect a
    // lock to be held while awaiting a future.
    histories: dashmap::DashMap<ChatId, Arc<std::sync::Mutex<Vec<ChatMessage>>>>,
    broadcasts: dashmap::DashMap<ChatId, tokio::sync::broadcast::Sender<ChatMessage>>,
}

impl ChatServer {
    pub fn new() -> Self {
        Self {
            histories: Default::default(),
            broadcasts: Default::default(),
        }
    }

    fn broadcast_message(&self, message: ChatMessage) {
        if let Some(sender) = self
            .broadcasts
            .get(&message.chat_id)
            .map(|r| r.value().clone())
        {
            // Intentionally ignoring errors here. If no receiver is interested
            // in the message any more, we don't care.
            #[allow(unused_must_use)]
            let _ = sender.send(message);
        }
    }

    pub fn send_message(&self, message: ChatMessage) -> Result<(), ChatServerErrors> {
        let shared_history = self.histories.entry(message.chat_id).or_default().clone();

        shared_history
            .lock()
            .map_err(|_| ChatServerErrors::lock_poisened("accessing history".to_string()))?
            .push(message.clone());

        self.broadcast_message(message);

        Ok(())
    }

    // Contract: If you got a stream by calling `join_chat`, you must call
    // `part_chat` after you dropped the stream. Otherwise some internal
    // state might not be cleaned up properly and a memory leak could
    // happen.
    //
    // Rationale: We could wrap the returned type to implement the Drop
    // trait and do everything `part_chat`, but I assume that would
    // require all sorts of Pin/Unpin shenanigans. Maybe we do that later.
    pub fn join_chat(&self, chat_id: ChatId) -> BroadcastStream<ChatMessage> {
        // Ensure the chat exists.
        self.histories.entry(chat_id).or_default();

        let receiver =
            if let Some(receiver) = self.broadcasts.get(&chat_id).map(|r| r.value().subscribe()) {
                receiver
            } else {
                let (sender, _) = tokio::sync::broadcast::channel(16);

                // Since our last dashmap operation, somebody could already inserted a sender for
                // this chat concurrently. So we only use our newly created channel, if the map
                // still doesn't have an entry for this chat. This is analogous to some concurrent
                // singleton double initialization pattern.
                self.broadcasts.entry(chat_id).or_insert(sender).subscribe()
            };
        return BroadcastStream::new(receiver);
    }

    pub fn part_chat(&self, chat_id: ChatId) {
        self.broadcasts
            .remove_if(&chat_id, |_, v| v.receiver_count() == 0);
    }

    pub fn get_chat_history(&self, chat_id: ChatId) -> Result<Vec<ChatMessage>, ChatServerErrors> {
        let history = self
            .histories
            .get(&chat_id)
            .ok_or_else(|| ChatServerErrors::chat_not_found(chat_id))?
            .lock()
            .map_err(|_| ChatServerErrors::lock_poisened("accessing history".to_string()))?
            .clone();
        Ok(history)
    }
}

#[derive(Debug, Error)]
pub enum ChatServerErrors {
    #[error("lock poison error: {message}")]
    LockPoisoned {
        backtrace: WrappedBacktrace,
        message: String,
    },
    #[error("chat {chat_id} not found")]
    ChatNotFound { chat_id: ChatId },
}

impl ChatServerErrors {
    pub fn lock_poisened(message: String) -> ChatServerErrors {
        ChatServerErrors::LockPoisoned {
            backtrace: WrappedBacktrace(Backtrace::capture()),
            message,
        }
    }
    pub fn chat_not_found(chat_id: ChatId) -> ChatServerErrors {
        ChatServerErrors::ChatNotFound { chat_id }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use futures::{FutureExt as _, future::try_join_all};
    use tokio_stream::StreamExt;

    use super::*;

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
        chat: ChatId,
        user: UserId,
        event_id: EventId,
    ) -> anyhow::Result<()> {
        let message = test_message(chat, user, event_id);
        sut.send_message(message)?;
        Ok(())
    }

    #[tokio::test]
    async fn concurrently_sending_messages_to_multiple_different_chats_the_correct_chats_should_receive_the_messages_and_the_histories_should_be_correct()
    -> anyhow::Result<()> {
        let sut = Arc::new(ChatServer::new());

        let user1 = UserId::random();
        let user2 = UserId::random();

        let chat1 = ChatId::random();
        let chat2 = ChatId::random();

        let mut receiver_for_chat1 = {
            let sut = sut.clone();
            sut.join_chat(chat1)
        };
        let mut receiver_for_chat2 = {
            let sut = sut.clone();
            sut.join_chat(chat2)
        };

        let event1_chat1 = EventId::random();
        let event2_chat1 = EventId::random();

        let event1_chat2 = EventId::random();
        let event2_chat2 = EventId::random();

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
        let chat_id = ChatId::random();
        {
            let receiver1 = sut.join_chat(chat_id);
            {
                let receiver2 = sut.join_chat(chat_id);
                assert_eq!(sut.broadcasts.len(), 1, "having two receivers the chat should have a broadcast map entry");
            }
            // dropped receiver2, now parting the chat
            sut.part_chat(chat_id);

            assert_eq!(sut.broadcasts.len(), 1, "having one receiver the chat should have a broadcast map entry");
        }
        // dropped recever1, now parting the chat
        sut.part_chat(chat_id);

        assert_eq!(sut.broadcasts.len(), 0, "having now receiver any more and having called `part_chat` triggering the cleanup, there should be no broadcast map entry any more");
    }
}
