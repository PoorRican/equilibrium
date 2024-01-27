use chrono::{DateTime, Utc};

/// A [`Message`] is a named event that is returned for logging.
///
/// A [`Message`] is meant to both be used for building graphs and for displaying a log to a user.
///
/// # Fields
/// * `name` - The name of the originating device
/// * `content` - The content of the message. This is a human-readable string that describes the
/// event that took place
/// * `timestamp` - The timestamp that the event took place
/// * `read_state` - Sensor read value (if applicable)
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
    /// Create a new message
    ///
    /// # Arguments
    /// * `name` - The name of the originating device
    /// * `content` - The content of the message
    /// * `timestamp` - The timestamp that the event took place
    /// * `read_state` - Sensor read value (if applicable)
    pub fn new(name: String, content: String, timestamp: DateTime<Utc>, read_state: Option<String>) -> Self {
        Self {
            name,
            content,
            timestamp,
            read_state
        }
    }

    pub fn get_controller_name(&self) -> String {
        self.name.clone()
    }
}