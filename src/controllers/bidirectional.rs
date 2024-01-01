/// Bidirectional Threshold

use crate::controllers::Controller;
use crate::types::Action;
use chrono::{DateTime, Duration, Utc};
use crate::input::Input;
use crate::output::Output;
use crate::scheduler::Scheduler;

enum State {
    BelowThreshold,
    WithinTolerance,
    AboveThreshold,
}

pub struct BidirectionalThreshold<I, O, O2>
    where
        I: Fn() -> String,
        O: FnMut(bool),
        O2: FnMut(bool),
{
    threshold: f32,
    tolerance: f32,
    input: Input<I>,
    increase_output: Output<O>,
    decrease_output: Output<O2>,
    interval: Duration,
    schedule: Scheduler,
}

impl<I, O, O2> BidirectionalThreshold<I, O, O2>
    where
        I: Fn() -> String,
        O: FnMut(bool),
        O2: FnMut(bool),
{
    pub fn new(
        threshold: f32,
        tolerance: f32,
        input: Input<I>,
        increase_output: Output<O>,
        decrease_output: Output<O2>,
        interval: Duration,
    ) -> Self {
        Self {
            threshold,
            tolerance,
            input,
            increase_output,
            decrease_output,
            interval,
            schedule: Scheduler::new(),
        }.schedule_first()
    }

    pub fn with_time(
        threshold: f32,
        tolerance: f32,
        input: Input<I>,
        increase_output: Output<O>,
        decrease_output: Output<O2>,
        interval: Duration,
        time: DateTime<Utc>,
    ) -> Self {
        let mut control = Self {
            threshold,
            tolerance,
            input,
            increase_output,
            decrease_output,
            interval,
            schedule: Scheduler::new(),
        };
        control.schedule_next(time);
        control
    }

    fn get_state(&mut self) -> State {
        let value = self.input.read().parse::<f32>().unwrap();
        if value > self.threshold + self.tolerance {
            State::AboveThreshold
        } else if value < self.threshold - self.tolerance {
            State::BelowThreshold
        } else {
            State::WithinTolerance
        }
    }

    fn handle_above_threshold(&mut self) {
        self.decrease_output.activate();
        self.increase_output.deactivate();
    }

    fn handle_below_threshold(&mut self) {
        self.increase_output.activate();
        self.decrease_output.deactivate();
    }

    fn handle_within_tolerance(&mut self) {
        self.increase_output.deactivate();
        self.decrease_output.deactivate();
    }

    fn schedule_next(&mut self, time: DateTime<Utc>) {
        self.schedule.schedule_read(time + self.interval);
    }

    fn schedule_first(mut self) -> Self {
        self.schedule_next(Utc::now());
        self
    }
}

impl<I, O, O2> Controller for BidirectionalThreshold<I, O, O2>
    where
        I: Fn() -> String,
        O: FnMut(bool),
        O2: FnMut(bool),
{
    fn poll(&mut self, time: DateTime<Utc>) {
        if let Some(event) = self.schedule.attempt_execution(time) {
            match event {
                Action::Read => {
                    let state = self.get_state();
                    match state {
                        State::AboveThreshold => self.handle_above_threshold(),
                        State::BelowThreshold => self.handle_below_threshold(),
                        State::WithinTolerance => self.handle_within_tolerance(),
                    }
                    self.schedule_next(time);
                }
                _ => {}
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
        let threshold = 10.0;
        let tolerance = 1.0;
        let input = Input::new(|| String::from("test"));

        let increase_output = Output::new(|_| {});
        let decrease_output = Output::new(|_| {});
        let interval = chrono::Duration::seconds(1);

        let controller = BidirectionalThreshold::new(
            threshold,
            tolerance,
            input,
            increase_output,
            decrease_output,
            interval,
        );

        assert_eq!(controller.threshold, threshold);
        assert_eq!(controller.tolerance, tolerance);
        assert_eq!(controller.interval, interval);

        assert!(controller.schedule.has_future_events());

        assert!(controller.increase_output.get_state().is_none());
        assert!(controller.decrease_output.get_state().is_none());
    }

    #[test]
    fn test_with_time() {
        let threshold = 10.0;
        let tolerance = 1.0;
        let input = Input::new(|| String::from("test"));

        let increase_output = Output::new(|_| {});
        let decrease_output = Output::new(|_| {});
        let interval = chrono::Duration::seconds(1);

        let time = Utc::now();
        let controller = BidirectionalThreshold::with_time(
            threshold,
            tolerance,
            input,
            increase_output,
            decrease_output,
            interval,
            time,
        );

        assert_eq!(controller.threshold, threshold);
        assert_eq!(controller.tolerance, tolerance);
        assert_eq!(controller.interval, interval);

        assert!(controller.schedule.has_future_events());
        let future_time = time + interval;
        assert_eq!(controller.schedule.get_future_events().get(0).unwrap().get_timestamp(), &future_time);
    }

    #[test]
    fn test_handle_above_threshold() {
        let threshold = 10.0;
        let tolerance = 1.0;
        let input = Input::new(|| String::from("test"));

        let increase_output = Output::new(|_| {});
        let decrease_output = Output::new(|_| {});
        let interval = Duration::seconds(1);

        let mut controller = BidirectionalThreshold::new(
            threshold,
            tolerance,
            input,
            increase_output,
            decrease_output,
            interval,
        ).schedule_first();

        controller.handle_above_threshold();

        assert_eq!(controller.increase_output.get_state(), Some(false));
        assert_eq!(controller.decrease_output.get_state(), Some(true));
    }

    #[test]
    fn test_handle_below_threshold() {
        let threshold = 10.0;
        let tolerance = 1.0;
        let input = Input::new(|| String::from("test"));

        let increase_output = Output::new(|_| {});
        let decrease_output = Output::new(|_| {});
        let interval = Duration::seconds(1);

        let mut controller = BidirectionalThreshold::new(
            threshold,
            tolerance,
            input,
            increase_output,
            decrease_output,
            interval,
        ).schedule_first();

        controller.handle_below_threshold();

        assert_eq!(controller.increase_output.get_state(), Some(true));
        assert_eq!(controller.decrease_output.get_state(), Some(false));
    }

    #[test]
    fn test_handle_within_tolerance() {
        let threshold = 10.0;
        let tolerance = 1.0;
        let input = Input::new(|| String::from(""));

        let increase_output = Output::new(|_| {});
        let decrease_output = Output::new(|_| {});
        let interval = Duration::seconds(1);

        let mut controller = BidirectionalThreshold::new(
            threshold,
            tolerance,
            input,
            increase_output,
            decrease_output,
            interval,
        ).schedule_first();

        controller.handle_within_tolerance();

        assert_eq!(controller.increase_output.get_state(), Some(false));
        assert_eq!(controller.decrease_output.get_state(), Some(false));
    }

    #[test]
    fn test_poll() {
        let threshold = 10.0;
        let tolerance = 1.0;

        let input_values = Arc::new(Mutex::new(VecDeque::from([
            "8.0".to_string(),
            "10.5".to_string(),
            "12.0".to_string(),
        ])));

        let input = Input::new(||
            input_values.lock().unwrap().pop_front().unwrap()
        );

        let increase_output = Output::new(|_| {});
        let decrease_output = Output::new(|_| {});
        let interval = Duration::seconds(1);

        let time = Utc::now();
        let mut controller = BidirectionalThreshold::with_time(
            threshold,
            tolerance,
            input,
            increase_output,
            decrease_output,
            interval,
            time,
        );

        // check default state
        assert!(controller.increase_output.get_state().is_none());
        assert!(controller.decrease_output.get_state().is_none());

        controller.poll(time);

        assert!(controller.increase_output.get_state().is_none());
        assert!(controller.decrease_output.get_state().is_none());

        // check before first read
        controller.poll(time + Duration::milliseconds(500));
        assert!(controller.increase_output.get_state().is_none());
        assert!(controller.decrease_output.get_state().is_none());

        // check first read which should be below threshold
        controller.poll(time + Duration::seconds(1));
        assert_eq!(controller.increase_output.get_state(), Some(true));
        assert_eq!(controller.decrease_output.get_state(), Some(false));

        // check again before second read
        controller.poll(time + Duration::seconds(1) + Duration::milliseconds(500));
        assert_eq!(controller.increase_output.get_state(), Some(true));
        assert_eq!(controller.decrease_output.get_state(), Some(false));

        // check second read which should be within tolerance
        controller.poll(time + Duration::seconds(2));
        assert_eq!(controller.increase_output.get_state(), Some(false));
        assert_eq!(controller.decrease_output.get_state(), Some(false));

        // check again before third read
        controller.poll(time + Duration::seconds(2) + Duration::milliseconds(500));
        assert_eq!(controller.increase_output.get_state(), Some(false));
        assert_eq!(controller.decrease_output.get_state(), Some(false));

        // check third read which should be above threshold
        controller.poll(time + Duration::seconds(3));
        assert_eq!(controller.increase_output.get_state(), Some(false));
        assert_eq!(controller.decrease_output.get_state(), Some(true));

        // check again after third read
        controller.poll(time + Duration::seconds(3) + Duration::milliseconds(500));
        assert_eq!(controller.increase_output.get_state(), Some(false));
        assert_eq!(controller.decrease_output.get_state(), Some(true));

    }
}