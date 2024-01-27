use chrono::{DateTime, Utc};
use crate::controllers::Controller;
use crate::types::Message;

/// A container for handling multiple controllers
///
/// This struct is used to multiple all controllers at once. The controllers are polled in the order
/// that they are added to the group, and the resulting [`Message`]s are returned.
///
/// Once a controller is added to the group, it is owned by the group and can only be accessed via
/// the `Controller` trait. This means that any other methods exposed by the controller are not
/// accessible.
pub struct ControllerGroup {
    controllers: Vec<Box<dyn Controller>>,
}

impl ControllerGroup {
    /// Create a new controller group
    pub fn new() -> Self {
        Self {
            controllers: Vec::new(),
        }
    }

    /// Builder method for adding a controller to the group
    ///
    /// This method takes ownership of the controller and adds it to the group. The controller can
    /// no longer be accessed directly, but can be polled via the [`Controller`] trait.
    ///
    /// # Arguments
    /// * `controller` - Any struct that implements the [`Controller`] trait
    ///
    /// # Returns
    /// The controller group with the new controller added
    ///
    /// # Example
    /// ```
    /// use equilibrium::controllers::{Controller, Threshold};
    ///
    /// let controller = Threshold::new(
    ///     70.0,
    ///     equilibrium::Input::default(),
    ///     equilibrium::Output::default(),
    ///     chrono::Duration::minutes(5),
    /// );
    ///
    /// let mut group = equilibrium::ControllerGroup::new()
    ///     .add_controller(controller);
    /// ```
    pub fn add_controller<C>(mut self, controller: C) -> Self
        where C: Controller + 'static
    {
        let wrapped = Box::new(controller);
        self.controllers.push(wrapped);
        self
    }

    /// Returns a reference to the controllers in the group
    ///
    /// This can be used for getting controller names or other information about the controllers
    /// which has been exposed via the [`Controller`] trait.
    pub fn get_controllers(&self) -> &Vec<Box<dyn Controller>> {
        &self.controllers
    }

    /// Poll all controllers in the group
    ///
    /// This method polls all controllers in the group and returns any resulting [`Message`]s
    /// in the order that the controllers were added to the group.
    ///
    /// # Arguments
    /// * `time` - The time to poll the controllers
    ///
    /// # Returns
    /// A vector of any [`Message`]s that were returned by the controllers. If no messages were
    /// returned, an empty vector is returned.
    pub fn poll(&mut self, time: DateTime<Utc>) -> Vec<Message> {
        let mut messages = Vec::new();
        for controller in self.controllers.iter_mut() {
            if let Some(message) = controller.poll(time) {
                messages.push(message);
            }
        }
        messages
    }
}


#[cfg(test)]
mod tests {
use super::*;
    use crate::controllers::{TimedOutput, Threshold};
    use crate::Output;
    use crate::Input;
    use chrono::{Duration, NaiveTime, TimeZone};

    #[test]
    fn test_new() {
        let group = ControllerGroup::new();

        assert_eq!(group.get_controllers().len(), 0);
    }

    #[test]
    fn test_add_controller() {
        // construct two different controllers
        let controller1 = TimedOutput::new(
            Output::default(),
            NaiveTime::from_hms_opt(5, 0, 0).unwrap(),
            Duration::hours(8),
        );

        let controller2 = Threshold::new(
            70.0,
            Input::default(),
            Output::default(),
            Duration::minutes(5),
        );

        // add controllers to group
        let group = ControllerGroup::new()
            .add_controller(controller1)
            .add_controller(controller2);

        // assert that the group has 2 controllers
        assert_eq!(group.get_controllers().len(), 2);
    }

    #[test]
    fn test_poll() {
        let now = Utc.with_ymd_and_hms(2021, 1, 1, 4, 59, 59).unwrap();

        // construct two different controllers and manually schedule first execution
        let timed_output_name = String::from("timed");
        let mut controller1 = TimedOutput::new_without_scheduled(
            Output::default(),
            NaiveTime::from_hms_opt(5, 0, 0).unwrap(),
            Duration::hours(12),
        ).schedule_first(now.clone());
        controller1.set_name(timed_output_name.clone());

        let threshold_name = String::from("threshold");
        let mut controller2 = Threshold::new_without_scheduled(
            70.0,
            Input::new(|| "69.0".to_string()),
            Output::default(),
            Duration::minutes(5),
        ).schedule_next(now.clone());
        controller2.set_name(threshold_name.clone());

        // construct controller
        let mut group = ControllerGroup::new()
            .add_controller(controller1)
            .add_controller(controller2);

        // begin to poll the group
        let messages = group.poll(now);
        assert_eq!(messages.len(), 0);      // no messages should be returned

        // poll again, this time the timed output should be activated
        let time = now + Duration::seconds(1);
        let messages = group.poll(time);
        assert_eq!(messages.len(), 1);      // one message should be returned
        assert_eq!(messages[0].get_controller_name(), timed_output_name);

        // poll again, this time the threshold output should be activated
        let time = time + Duration::minutes(5);
        let messages = group.poll(time);
        assert_eq!(messages.len(), 1);      // one message should be returned
        assert_eq!(messages[0].get_controller_name(), threshold_name);

        // poll again when both controllers should be activated
        let time = Utc.with_ymd_and_hms(2021, 1, 1, 17, 0, 0).unwrap();
        let messages = group.poll(time);
        assert_eq!(messages.len(), 2);      // two messages should be returned
        assert_eq!(messages[0].get_controller_name(), timed_output_name);
        assert_eq!(messages[1].get_controller_name(), threshold_name);
    }
}