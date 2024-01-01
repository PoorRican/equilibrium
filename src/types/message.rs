use chrono::{DateTime, Utc};
use crate::types::{Action, Event};

/// A `Message` is a named event that is returned for logging
#[derive(Debug, PartialEq, Clone)]
pub struct Message {
    /// The name of the originating device
    name: String,

    /// The action of the event
    action: Action,

    /// The value of the read value
    value: Option<String>,

    /// The timestamp that the event took place
    timestamp: DateTime<Utc>,
}

impl Message {
    fn from_event(event: Event, name: String) -> Self {
        Self {
            name,
            action: event.get_action(),
            value: event.get_value().clone(),
            timestamp: event.get_timestamp().clone(),
        }
    }
}