use chrono::{DateTime, Duration, Utc};
use crate::controllers::Controller;
use crate::input::Input;
use crate::output::Output;
use crate::scheduler::Scheduler;

pub struct Threshold<I, O>
where
    I: Fn() -> String,
    O: FnMut(bool),
{
    threshold: f32,
    input: Input<I>,
    output: Output<O>,
    interval: Duration,
    schedule: Scheduler,
    inverted: bool,
}

impl<I, O> Threshold<I, O>
where
    I: Fn() -> String,
    O: FnMut(bool),
{
    pub fn new(threshold: f32, input: Input<I>, output: Output<O>, interval: Duration) -> Threshold<I, O> {
        Threshold {
            threshold,
            input,
            output,
            schedule: Scheduler::new(),
            interval,
            inverted: false,
        }.schedule_first()
    }

    pub fn with_time(threshold: f32, input: Input<I>, output: Output<O>, interval: Duration, time: DateTime<Utc>) -> Threshold<I, O> {
        let mut controller = Threshold {
            threshold,
            input,
            output,
            schedule: Scheduler::new(),
            interval,
            inverted: false,
        };
        controller.schedule_next(time);
        controller
    }

    pub fn set_inverted(mut self) -> Self {
        self.inverted = true;
        self
    }

    pub fn get_threshold(&self) -> f32 {
        self.threshold
    }

    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold;
    }

    /// Read the input and return true if the value is above the threshold
    fn above_threshold(&mut self) -> bool {
        let value = self.input.read();
        let value = value.parse::<f32>().unwrap();
        if value > self.threshold {
            true
        } else {
            false
        }
    }

    fn handle_above_threshold(&mut self) {
        match self.inverted {
            true => self.output.deactivate(),
            false => self.output.activate(),
        }
    }

    fn handle_below_threshold(&mut self) {
        match self.inverted {
            true => self.output.activate(),
            false => self.output.deactivate(),
        }
    }

    fn schedule_next(&mut self, time: DateTime<Utc>) {
        self.schedule.schedule_read(time + self.interval);
    }

    fn schedule_first(mut self) -> Self {
        self.schedule_next(Utc::now());
        self
    }
}

impl<I, O> Controller for Threshold<I, O>
    where
        I: Fn() -> String,
        O: FnMut(bool),
{
    fn poll(&mut self, time: DateTime<Utc>) {
        if let Some(action) = self.schedule.attempt_execution(time) {
            match action {
                crate::types::Action::Read => {
                    // Read the input and handle the result
                    match self.above_threshold() {
                        true => self.handle_above_threshold(),
                        false => self.handle_below_threshold()
                    }

                    // Schedule the next read
                    self.schedule.schedule_read(time + self.interval);
                },
                _ => panic!("Encountered unexpected action in threshold controller")
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};
    use super::*;

    #[test]
    fn test_new() {
        let threshold = 0.0;
        let interval = Duration::seconds(1);

        let input = Input::new(|| String::from("0.0"));
        let output = Output::new(|_| {});
        let controller = Threshold::new(
            threshold,
            input,
            output,
            interval,
        );

        assert_eq!(controller.get_threshold(), 0.0);
        assert_eq!(controller.inverted, false);
        assert_eq!(controller.interval, Duration::seconds(1));
        assert!(controller.schedule.has_future_events());
    }

    #[test]
    fn test_with_time() {
        let threshold = 0.0;
        let interval = Duration::seconds(1);

        let input = Input::new(|| String::from("0.0"));
        let output = Output::new(|_| {});
        let time = Utc::now();
        let controller = Threshold::with_time(
            threshold,
            input,
            output,
            interval,
            time,
        );

        assert_eq!(controller.get_threshold(), 0.0);
        assert_eq!(controller.inverted, false);
        assert_eq!(controller.interval, Duration::seconds(1));
        assert!(controller.schedule.has_future_events());
        let future_time = time + interval;
        assert_eq!(controller.schedule.get_future_events().get(0).unwrap().get_timestamp(), &future_time);
    }

    #[test]
    fn test_set_inverted() {
        // check default
        let input = Input::new(|| String::from("0.0"));
        let output = Output::new(|_| {});
        let controller = Threshold::new(
            0.0,
            input,
            output,
            Duration::seconds(1)
        );

        assert_eq!(controller.inverted, false);

        // check after setting
        let input = Input::new(|| String::from("0.0"));
        let output = Output::new(|_| {});
        let controller = Threshold::new(
            0.0,
            input,
            output,
            Duration::seconds(1)
        ).set_inverted();

        assert_eq!(controller.inverted, true);
    }

    #[test]
    fn test_get_threshold() {
        let threshold = 5.0;
        let input = Input::new(|| String::from("0.0"));
        let output = Output::new(|_| {});
        let controller = Threshold::new(
            threshold,
            input,
            output,
            Duration::seconds(1)
        );

        assert_eq!(controller.get_threshold(), threshold);
    }

    #[test]
    fn test_set_threshold() {
        let threshold = 5.0;
        let input = Input::new(|| String::from("0.0"));
        let output = Output::new(|_| {});
        let mut controller = Threshold::new(
            threshold,
            input,
            output,
            Duration::seconds(1)
        );

        assert_eq!(controller.get_threshold(), threshold);

        let new_threshold = 10.0;
        controller.set_threshold(new_threshold);
        assert_eq!(controller.get_threshold(), new_threshold);
    }

    #[test]
    fn test_above_threshold() {
        // check when below threshold
        let input = Input::new(|| String::from("0.0"));
        let output = Output::new(|_| {});
        let mut controller = Threshold::new(
            5.0,
            input,
            output,
            Duration::seconds(1)
        );

        assert_eq!(controller.above_threshold(), false);

        // check when above threshold
        let input = Input::new(|| String::from("10.0"));
        let output = Output::new(|_| {});
        let mut controller = Threshold::new(
            5.0,
            input,
            output,
            Duration::seconds(1)
        );

        assert_eq!(controller.above_threshold(), true);
    }

    #[test]
    fn tests_handle_above_threshold() {
        // check when not inverted
        let input = Input::new(|| String::from("10.0"));

        let external_output_state = Arc::new(Mutex::new(false));
        let output = Output::new(|state| {
            let mut external_state = external_output_state.lock().unwrap();
            *external_state = state;
        });
        let mut controller = Threshold::new(
            5.0,
            input,
            output,
            Duration::seconds(1)
        );

        assert_eq!(external_output_state.lock().unwrap().clone(), false);
        controller.handle_above_threshold();
        assert_eq!(external_output_state.lock().unwrap().clone(), true);

        // check when inverted
        let mut controller = controller.set_inverted();

        assert_eq!(external_output_state.lock().unwrap().clone(), true);
        controller.handle_above_threshold();
        assert_eq!(external_output_state.lock().unwrap().clone(), false);
    }

    #[test]
    fn test_handle_below_threshold() {
        // check when not inverted
        let input = Input::new(|| String::from("0.0"));

        let external_output_state = Arc::new(Mutex::new(true));
        let output = Output::new(|state| {
            let mut external_state = external_output_state.lock().unwrap();
            *external_state = state;
        });

        let mut controller = Threshold::new(
            5.0,
            input,
            output,
            Duration::seconds(1)
        );

        assert_eq!(external_output_state.lock().unwrap().clone(), true);
        controller.handle_below_threshold();
        assert_eq!(external_output_state.lock().unwrap().clone(), false);
    }

    #[test]
    fn test_poll_not_inverted() {
        let state_sequence =
            Arc::new(
                Mutex::new(
                    VecDeque::from([
                        "0.0".to_string(),
                        "10.0".to_string(),
                        "0.0".to_string(),
                    ])));
        let input = Input::new(|| {
            let mut state_sequence = state_sequence.lock().unwrap();
            state_sequence.pop_front().unwrap()
        });

        let external_output_state = Arc::new(Mutex::new(false));
        let output = Output::new(|state| {
            let mut external_state = external_output_state.lock().unwrap();
            *external_state = state;
        });

        let time = Utc::now();

        let mut controller = Threshold::with_time(
            5.0,
            input,
            output,
            Duration::seconds(1),
            time
        );

        // check default state
        assert_eq!(external_output_state.lock().unwrap().clone(), false);

        // check before first read
        controller.poll(time + Duration::milliseconds(500));
        assert_eq!(external_output_state.lock().unwrap().clone(), false);

        // check after first read when below threshold
        controller.poll(time + Duration::seconds(1));
        assert_eq!(external_output_state.lock().unwrap().clone(), false);

        // check before second poll execution
        controller.poll(time + Duration::milliseconds(1500));
        assert_eq!(external_output_state.lock().unwrap().clone(), false);

        // check after second read when above threshold
        controller.poll(time + Duration::seconds(2));
        assert_eq!(external_output_state.lock().unwrap().clone(), true);

        // check after second read before third read
        controller.poll(time + Duration::microseconds(2500));
        assert_eq!(external_output_state.lock().unwrap().clone(), true);

        // check after third read when below threshold
        controller.poll(time + Duration::seconds(3));
        assert_eq!(external_output_state.lock().unwrap().clone(), false);
    }

    #[test]
    fn test_poll_inverted() {
        let state_sequence =
            Arc::new(
                Mutex::new(
                    VecDeque::from([
                        "0.0".to_string(),
                        "10.0".to_string(),
                        "0.0".to_string(),
                    ])));
        let input = Input::new(|| {
            let mut state_sequence = state_sequence.lock().unwrap();
            state_sequence.pop_front().unwrap()
        });

        let external_output_state = Arc::new(Mutex::new(false));
        let output = Output::new(|state| {
            let mut external_state = external_output_state.lock().unwrap();
            *external_state = state;
        });

        let time = Utc::now();

        let mut controller = Threshold::with_time(
            5.0,
            input,
            output,
            Duration::seconds(1),
            time
        ).set_inverted();

        // check default state
        assert_eq!(external_output_state.lock().unwrap().clone(), false);

        // check before first read
        controller.poll(time + Duration::milliseconds(500));
        assert_eq!(external_output_state.lock().unwrap().clone(), false);

        // check after first read when below threshold
        controller.poll(time + Duration::seconds(1));
        assert_eq!(external_output_state.lock().unwrap().clone(), true);

        // check before second poll execution
        controller.poll(time + Duration::milliseconds(1500));
        assert_eq!(external_output_state.lock().unwrap().clone(), true);

        // check after second read when above threshold
        controller.poll(time + Duration::seconds(2));
        assert_eq!(external_output_state.lock().unwrap().clone(), false);

        // check after second read before third read
        controller.poll(time + Duration::microseconds(2500));
        assert_eq!(external_output_state.lock().unwrap().clone(), false);

        // check after third read when below threshold
        controller.poll(time + Duration::seconds(3));
        assert_eq!(external_output_state.lock().unwrap().clone(), true);
    }
}