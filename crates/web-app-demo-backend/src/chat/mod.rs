use std::{backtrace::Backtrace, sync::Arc};

use thiserror::Error;
use tokio_stream::wrappers::BroadcastStream;

use crate::util::wrappedbacktrace::WrappedBacktrace;

pub mod models;

#[allow(dead_code)]
pub struct ChatServer {
    // We intentionally use a std::sync::Mutex, as we never expect a
    // lock to be held while awaiting a future.
    histories: dashmap::DashMap<models::ChatId, Arc<std::sync::Mutex<Vec<models::ChatMessage>>>>,
    broadcasts:
        dashmap::DashMap<models::ChatId, tokio::sync::broadcast::Sender<models::ChatMessage>>,
}

#[allow(dead_code)]
impl ChatServer {
    pub fn new() -> Self {
        Self {
            histories: Default::default(),
            broadcasts: Default::default(),
        }
    }

    fn broadcast_message(&self, message: models::ChatMessage) {
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

    pub fn send_message(&self, message: models::ChatMessage) -> Result<(), ChatServerErrors> {
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
    pub fn join_chat(&self, chat_id: models::ChatId) -> BroadcastStream<models::ChatMessage> {
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

    pub fn part_chat(&self, chat_id: models::ChatId) {
        self.broadcasts
            .remove_if(&chat_id, |_, v| v.receiver_count() == 0);
    }

    pub fn get_chat_history(
        &self,
        chat_id: models::ChatId,
    ) -> Result<Vec<models::ChatMessage>, ChatServerErrors> {
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
    ChatNotFound { chat_id: models::ChatId },
}

impl ChatServerErrors {
    pub fn lock_poisened(message: String) -> ChatServerErrors {
        ChatServerErrors::LockPoisoned {
            backtrace: WrappedBacktrace(Backtrace::capture()),
            message,
        }
    }
    pub fn chat_not_found(chat_id: models::ChatId) -> ChatServerErrors {
        ChatServerErrors::ChatNotFound { chat_id }
    }
}

#[cfg(test)]
mod tests;
