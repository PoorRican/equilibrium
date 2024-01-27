use chrono::{DateTime, Utc};
use crate::controllers::Controller;
use crate::types::Message;

pub struct ControllerGroup {
    controllers: Vec<Box<dyn Controller>>,
}

impl ControllerGroup {
    pub fn new() -> Self {
        Self {
            controllers: Vec::new(),
        }
    }

    pub fn add_controller<C>(&mut self, controller: C) where C: Controller + 'static {
        let wrapped = Box::new(controller);
        self.controllers.push(wrapped);
    }

    pub fn get_controllers(&self) -> &Vec<Box<dyn Controller>> {
        &self.controllers
    }

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
        let mut group = ControllerGroup::new();

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
        group.add_controller(controller1);
        group.add_controller(controller2);

        // assert that the group has 2 controllers
        assert_eq!(group.get_controllers().len(), 2);
    }

    #[test]
    fn test_poll() {
        let mut group = ControllerGroup::new();

        let now = Utc.with_ymd_and_hms(2021, 1, 1, 4, 59, 59).unwrap();

        // construct two different controllers
        let timed_output_name = String::from("timed");
        let mut controller1 = TimedOutput::new(
            Output::default(),
            NaiveTime::from_hms_opt(5, 0, 0).unwrap(),
            Duration::hours(12),
        );
        controller1.set_name(timed_output_name.clone());
        let controller1 = controller1.schedule_first(now.clone());

        let threshold_name = String::from("threshold");
        let mut controller2 = Threshold::new(
            70.0,
            Input::new(|| "69.0".to_string()),
            Output::default(),
            Duration::minutes(5),
        );
        controller2.set_name(threshold_name.clone());
        let controller2 = controller2.schedule_next(now.clone());

        // add controllers to group
        group.add_controller(controller1);
        group.add_controller(controller2);

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