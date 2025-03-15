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
    use futures::FutureExt as _;
    use tokio_stream::StreamExt;

    use super::*;

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
        let message = ChatMessage {
            event_id: EventId::random(),
            timestamp: ChatTimestamp::epoch(),
            chat_id,
            user_id: UserId::random(),
            display_name: DisplayName::new("Hugo".to_string()),
            message: Message::new("Hello!".to_string()),
        };

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
        let event_id = EventId::random();
        let message = ChatMessage {
            event_id,
            timestamp: ChatTimestamp::epoch(),
            chat_id,
            user_id: UserId::random(),
            display_name: DisplayName::new("Hugo".to_string()),
            message: Message::new("Hello!".to_string()),
        };
        let mut receiver = sut.join_chat(chat_id);
        sut.send_message(message)
            .context("sending a message should succeed")?;
        if let Some(message) = receiver
            .try_next()
            .now_or_never()
            .context("after sending a message to a chat, it should be immediately available")?
            .context("receiving messages for a chat should succeed")?
        {
            assert_eq!(
                event_id, message.event_id,
                "the message received from the chat should be the message sent in the test"
            )
        }

        assert!(
            receiver.try_next().now_or_never().is_none(),
            "there shouldn't be any other messages for the chat, but the one sent"
        );

        Ok(())
    }
}
