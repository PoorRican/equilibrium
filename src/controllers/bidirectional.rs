/// Bidirectional Threshold

use crate::controllers::Controller;
use crate::types::{Action, Message};
use chrono::{DateTime, Duration, Utc};
use crate::input::Input;
use crate::output::Output;
use crate::scheduler::Scheduler;

/// Internal state of the controller
///
/// This is used to determine which output should be activated.
enum State {
    BelowThreshold,
    WithinTolerance,
    AboveThreshold,
}

/// Controller with two outputs that are activated when the input is above or below a threshold.
///
/// This is used to control a system that has two modes of control (increase and decrease). This controller is
/// not very precise, and is intended to keep a value within a general range.
///
/// ## Potential Use Cases
/// * For a reservoir or sump pump, turning on a pump or relief valve according to fill level
///
/// # Example
/// In this example, the controller will increase the value if it rises above 11.0, and decrease the value if it falls
/// below 9.0.
/// ```
/// use chrono::{Duration, Utc};
/// use equilibrium::controllers::{Controller, BidirectionalThreshold};
/// use equilibrium::input::Input;
/// use equilibrium::output::Output;
///
/// let threshold = 10.0;
/// let tolerance = 1.0;
///
/// let interval = Duration::seconds(1);
///
/// let mut controller = BidirectionalThreshold::with_first(
///     threshold,
///     tolerance,
///     Input::default(),
///     Output::default(),
///     Output::default(),
///     interval,
/// );
///
/// controller.poll(Utc::now());
/// ```
#[derive(Debug)]
pub struct BidirectionalThreshold<I, O, O2>
    where
        I: Fn() -> String,
        O: FnMut(bool),
        O2: FnMut(bool),
{
    name: Option<String>,
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
    /// Create a new controller without scheduling the first read
    ///
    /// [`BidirectionalThreshold::schedule_next()`] must be called after this function.
    ///
    /// [`BidirectionalThreshold::with_first()`] is the preferred API for instantiation.
    pub fn new(
        threshold: f32,
        tolerance: f32,
        input: Input<I>,
        increase_output: Output<O>,
        decrease_output: Output<O2>,
        interval: Duration,
    ) -> Self {
        Self {
            name: None,
            threshold,
            tolerance,
            input,
            increase_output,
            decrease_output,
            interval,
            schedule: Scheduler::new(),
        }
    }

    /// Create a new controller with a specific time as the first read time
    ///
    /// This is the recommended API for instantiation.
    pub fn with_first(
        threshold: f32,
        tolerance: f32,
        input: Input<I>,
        increase_output: Output<O>,
        decrease_output: Output<O2>,
        interval: Duration,
    ) -> Self {
        Self {
            name: None,
            threshold,
            tolerance,
            input,
            increase_output,
            decrease_output,
            interval,
            schedule: Scheduler::new(),
        }.schedule_next(None)
    }

    /// Read the input and determine the state of the controller
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

    /// Attempt to lower the input value
    fn handle_above_threshold(&mut self) {
        self.decrease_output.activate();
        self.increase_output.deactivate();
    }

    /// Attempt to raise the input value
    fn handle_below_threshold(&mut self) {
        self.increase_output.activate();
        self.decrease_output.deactivate();
    }

    /// Turn off both outputs to maintain the current value
    fn handle_within_tolerance(&mut self) {
        self.increase_output.deactivate();
        self.decrease_output.deactivate();
    }

    /// Schedule the next read for the specified time
    fn schedule_next_in_place(&mut self, time: DateTime<Utc>) {
        self.schedule.schedule_read(time + self.interval);
    }

    /// Builder method to schedule the next read for the specified time
    ///
    /// If no time is specified, the current time will be used.
    pub fn schedule_next<T>(mut self, time: T) -> Self
    where T: Into<Option<DateTime<Utc>>>{
        let time= time.into().unwrap_or_else(|| Utc::now());
        self.schedule_next_in_place(time);
        self
    }
}

impl<I, O, O2> Controller for BidirectionalThreshold<I, O, O2>
    where
        I: Fn() -> String,
        O: FnMut(bool),
        O2: FnMut(bool),
{
    fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    fn get_name(&self) -> Option<String> {
        self.name.clone()
    }

    fn poll(&mut self, time: DateTime<Utc>) -> Option<Message> {
        if let Some(event) = self.schedule.attempt_execution(time) {
            match event.get_action() {
                Action::Read => {
                    let msg = match self.get_state() {
                        State::AboveThreshold => {
                            self.handle_above_threshold();
                            "Above Threshold".to_string()
                        },
                        State::BelowThreshold => {
                            self.handle_below_threshold();
                            "Below Threshold".to_string()
                        },
                        State::WithinTolerance => {
                            self.handle_within_tolerance();
                            "Within Tolerance".to_string()
                        },
                    };
                    self.schedule_next_in_place(time);

                    let read_state = self.input.get_state().clone();
                    return Some(Message::new(
                        self.get_name().unwrap_or_default(),
                        msg,
                        event.get_timestamp().clone(),
                        read_state,
                    ));
                }
                _ => {}
            }
        }
        None
    }
}

impl Default for BidirectionalThreshold<fn() -> String, fn(bool), fn(bool)> {
    fn default() -> Self {
        Self::new(
            0.0,
            0.0,
            Input::default(),
            Output::default(),
            Output::default(),
            Duration::seconds(1),
        )
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
        let input = Input::default();

        let increase_output = Output::default();
        let decrease_output = Output::default();
        let interval = Duration::seconds(1);

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

        assert!(!controller.schedule.has_future_events());

        assert!(controller.increase_output.get_state().is_none());
        assert!(controller.decrease_output.get_state().is_none());
    }

    #[test]
    fn test_with_time() {
        let threshold = 10.0;
        let tolerance = 1.0;
        let input = Input::default();

        let increase_output = Output::default();
        let decrease_output = Output::default();
        let interval = chrono::Duration::seconds(1);

        let controller = BidirectionalThreshold::with_first(
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
    }

    #[test]
    fn test_handle_above_threshold() {
        let threshold = 10.0;
        let tolerance = 1.0;
        let input = Input::default();

        let increase_output = Output::default();
        let decrease_output = Output::default();
        let interval = Duration::seconds(1);

        let mut controller = BidirectionalThreshold::new(
            threshold,
            tolerance,
            input,
            increase_output,
            decrease_output,
            interval,
        );

        controller.handle_above_threshold();

        assert_eq!(controller.increase_output.get_state(), Some(false));
        assert_eq!(controller.decrease_output.get_state(), Some(true));
    }

    #[test]
    fn test_handle_below_threshold() {
        let threshold = 10.0;
        let tolerance = 1.0;
        let input = Input::default();

        let increase_output = Output::default();
        let decrease_output = Output::default();
        let interval = Duration::seconds(1);

        let mut controller = BidirectionalThreshold::new(
            threshold,
            tolerance,
            input,
            increase_output,
            decrease_output,
            interval,
        );

        controller.handle_below_threshold();

        assert_eq!(controller.increase_output.get_state(), Some(true));
        assert_eq!(controller.decrease_output.get_state(), Some(false));
    }

    #[test]
    fn test_handle_within_tolerance() {
        let threshold = 10.0;
        let tolerance = 1.0;
        let input = Input::default();

        let increase_output = Output::default();
        let decrease_output = Output::default();
        let interval = Duration::seconds(1);

        let mut controller = BidirectionalThreshold::new(
            threshold,
            tolerance,
            input,
            increase_output,
            decrease_output,
            interval,
        );

        controller.handle_within_tolerance();

        assert_eq!(controller.increase_output.get_state(), Some(false));
        assert_eq!(controller.decrease_output.get_state(), Some(false));
    }

    #[test]
    fn test_get_set_name() {
        let mut controller = BidirectionalThreshold::default();

        assert_eq!(controller.get_name(), None);

        controller.set_name(String::from("test"));

        assert_eq!(controller.get_name(), Some(String::from("test")));
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

        let increase_output = Output::default();
        let decrease_output = Output::default();
        let interval = Duration::seconds(1);

        let time = Utc::now();
        let mut controller = BidirectionalThreshold::new(
            threshold,
            tolerance,
            input,
            increase_output,
            decrease_output,
            interval,
        ).schedule_next(time);

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