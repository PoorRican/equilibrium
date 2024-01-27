use chrono::{DateTime, Utc};
use crate::types::action::Action;

/// Encapsulate IO events for scheduling or logging
///
/// IO Events occur in the future and are used by [`crate::controllers::Controller`]s to determine
/// when to execute IO actions.
///
/// The associated [`Action`] is expected to be handled by the [`crate::controllers::Controller`]
/// at the time specified by the timestamp.
#[derive(Debug, PartialEq, Clone)]
pub struct Event {
    action: Action,

    /// Used during read events to store the value read
    value: Option<String>,

    timestamp: DateTime<Utc>,
}

impl Event {
    /// Create a new event
    ///
    /// # Arguments
    /// * `action` - The action to be executed at the specified time
    /// * `timestamp` - The time at which the action should be executed
    pub fn new(action: Action, timestamp: DateTime<Utc>) -> Self {
        Self {
            action,
            value: None,
            timestamp,
        }
    }

    /// Returns true if the event should be executed at the specified time
    pub fn should_execute(&self, time: DateTime<Utc>) -> bool {
        if self.timestamp > time {
            false
        } else {
            true
        }
    }

    /// Returns the action associated with the event
    pub fn get_action(&self) -> Action {
        self.action
    }

    /// Returns the value associated with the event
    ///
    /// This is only used during read events
    pub fn get_value(&self) -> &Option<String> {
        &self.value
    }

    /// Returns the timestamp associated with the event
    pub fn get_timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    /// Set the value associated with the event
    ///
    /// This is only used during read events
    pub fn set_value(&mut self, value: String) {
        self.value = Some(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    #[test]
    fn test_new() {

        let timestamp = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0)
            .unwrap();
        let event = Event::new(Action::Read, timestamp);

        assert_eq!(event.get_action(), Action::Read);
        assert_eq!(event.get_timestamp(), &timestamp);
        assert!(event.get_value().is_none());
    }

    #[test]
    fn test_should_execute() {
        let timestamp = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0)
            .unwrap();
        let event = Event::new(Action::Read, timestamp);

        let time = Utc.with_ymd_and_hms(2022, 1, 1, 0, 0, 0)
            .unwrap();
        assert_eq!(event.should_execute(time), false);

        let time = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0)
            .unwrap();
        assert_eq!(event.should_execute(time), true);

        let time = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0)
            .unwrap();
        assert_eq!(event.should_execute(time), true);
    }
}