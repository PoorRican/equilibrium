use chrono::{DateTime, Utc};
use crate::types::{Action, Event};

/// A `Message` is a named event that is returned for logging
#[derive(Debug, PartialEq, Clone)]
pub struct Message {
    /// The name of the originating device
    name: String,

    /// The content of the message
    content: String,

    /// The timestamp that the event took place
    timestamp: DateTime<Utc>,
}

impl Message {
    fn new(name: String, content: String, timestamp: DateTime<Utc>) -> Self {
        Self {
            name,
            content,
            timestamp,
        }
    }
}