use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UserId(uuid::Uuid);

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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
