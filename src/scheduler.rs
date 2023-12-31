use chrono::{DateTime, Utc};
use crate::types::action::Action;
use crate::types::event::Event;

/// A way to manage future and past [`Event`]s
///
/// The purpose of this struct is to manage when IO events should be executed. "Scheduling" of
/// events should be handled outside of this struct. This struct should only be used to determine
/// when an event should be executed.
pub struct Scheduler {
    future_events: Vec<Event>,
    events: Vec<Event>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            future_events: Vec::new(),
            events: Vec::new(),
        }
    }

    /// Returns true if there are any future events
    pub fn has_future_events(&self) -> bool {
        !self.future_events.is_empty()
    }

    /// Schedule an `On` event for the specified time
    pub fn schedule_on(&mut self, timestamp: DateTime<Utc>) {
        let event = Event::new(Action::On, timestamp);
        self.future_events.push(event);
    }

    /// Schedule an `Off` event for the specified time
    pub fn schedule_off(&mut self, timestamp: DateTime<Utc>) {
        let event = Event::new(Action::Off, timestamp);
        self.future_events.push(event);
    }

    /// Schedule a `Read` event for the specified time
    pub fn schedule_read(&mut self, timestamp: DateTime<Utc>) {
        let event = Event::new(Action::Read, timestamp);
        self.future_events.push(event);
    }

    /// Attempt to execute any events that should be executed at the specified time
    ///
    /// Any event that should be executed at the specified time will be removed from the list of
    /// future events and added to the list of past events. The action associated with the event
    /// will be returned.
    ///
    /// If no events should be executed at the specified time, `None` will be returned.
    ///
    /// # Arguments
    /// * `time` - The time to check for events that should be executed
    ///
    /// # Returns
    /// * `Some(Action)` - The action associated with the event that should be executed
    /// * `None` - No events should be executed at the specified time
    pub fn attempt_execution(&mut self, time: DateTime<Utc>) -> Option<Action> {
        if let Some(index) = self.future_events.iter().position(|e| e.should_execute(time)) {
            let event = self.future_events.remove(index);
            let action = event.get_action();
            self.events.push(event);
            Some(action)
        } else {
            None
        }
    }
}


#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use super::*;

    #[test]
    fn test_new() {
        let scheduler = Scheduler::new();

        assert_eq!(scheduler.has_future_events(), false);
    }

    #[test]
    fn test_schedule_on() {
        let mut scheduler = Scheduler::new();
        let timestamp = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0)
            .unwrap();
        scheduler.schedule_on(timestamp);

        assert_eq!(scheduler.has_future_events(), true);
    }

    #[test]
    fn test_schedule_off() {
        let mut scheduler = Scheduler::new();
        let timestamp = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0)
            .unwrap();
        scheduler.schedule_off(timestamp);

        assert_eq!(scheduler.has_future_events(), true);
    }

    #[test]
    fn test_schedule_read() {
        let mut scheduler = Scheduler::new();
        let timestamp = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0)
            .unwrap();
        scheduler.schedule_read(timestamp);

        assert_eq!(scheduler.has_future_events(), true);
    }

    #[test]
    fn test_attempt_execution() {
        let mut scheduler = Scheduler::new();
        let timestamp = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0)
            .unwrap();
        scheduler.schedule_on(timestamp);

        let timestamp = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 1)
            .unwrap();
        let action = scheduler.attempt_execution(timestamp);

        assert_eq!(action, Some(Action::On));
        assert_eq!(scheduler.has_future_events(), false);
        assert_eq!(scheduler.events.len(), 1);

        // test two events
        let timestamp = Utc.with_ymd_and_hms(2023, 1, 1, 0, 1, 0)
            .unwrap();
        scheduler.schedule_off(timestamp);

        let timestamp = Utc.with_ymd_and_hms(2023, 1, 1, 0, 3, 0)
            .unwrap();
        scheduler.schedule_on(timestamp);

        let timestamp = Utc.with_ymd_and_hms(2023, 1, 1, 0, 2, 0)
            .unwrap();
        let action = scheduler.attempt_execution(timestamp);

        assert_eq!(action, Some(Action::Off));
        assert_eq!(scheduler.has_future_events(), true);
        assert_eq!(scheduler.events.len(), 2);

        let timestamp = Utc.with_ymd_and_hms(2023, 1, 1, 0, 4, 0)
            .unwrap();
        let action = scheduler.attempt_execution(timestamp);

        assert_eq!(action, Some(Action::On));
        assert_eq!(scheduler.has_future_events(), false);
        assert_eq!(scheduler.events.len(), 3);
    }
}