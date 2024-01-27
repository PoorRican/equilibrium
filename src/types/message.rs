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

    /// Sensor read value
    read_state: Option<String>,
}

impl Message {
   pub fn new(name: String, content: String, timestamp: DateTime<Utc>, read_state: Option<String>) -> Self {
        Self {
            name,
            content,
            timestamp,
            read_state
        }
    }
}