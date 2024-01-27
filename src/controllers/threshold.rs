use chrono::{DateTime, Duration, Utc};
use crate::controllers::Controller;
use crate::input::Input;
use crate::output::Output;
use crate::scheduler::Scheduler;
use crate::types::Message;

/// A controller that reads an input and activates an output if the value is above or below a threshold
///
/// This controller is not very precise as it has no ability to prevent overcompensation. If control
/// is required to be more precise, and has the ability for two outputs, consider using the
/// [`BidirectionalThreshold`](crate::controllers::BidirectionalThreshold) controller. If that is not
/// possible, then consider reducing the polling interval.
///
/// ## Operation
/// By default, this controller will activate the output when the input is above the threshold and
/// deactivate the output when the input is below the threshold. Use [`set_inverted`](Threshold::set_inverted)
/// so that the output is activated when the input is below the threshold and deactivated when the
/// input is above the threshold.
///
///
/// ## Potential Use Cases
/// * Controlling a fan based on temperature
/// * Controlling CO2 levels in a grow room
/// * Maintaining sufficient water levels in a reservoir
///
///
/// # Example
///
/// In this example, the controller will actuate the output if it rises above 10.0.
/// ```
/// use chrono::{Duration, Utc};
/// use equilibrium::controllers::{Controller, Threshold};
/// use equilibrium::Input;
/// use equilibrium::Output;
///
/// let threshold = 10.0;
///
/// let interval = Duration::seconds(1);
///
/// let mut controller = Threshold::new(
///     threshold,
///     Input::default(),
///     Output::default(),
///     interval,
/// );
///
/// controller.poll(Utc::now());
/// ```
///
///
/// The output can be inverted to actuate the output if it falls below the threshold.
/// In this example, the output is actuated when the threshold falls below 10.0
/// ```
/// use chrono::{Duration, Utc};
/// use equilibrium::controllers::{Controller, Threshold};
/// use equilibrium::Input;
/// use equilibrium::Output;
///
/// let threshold = 10.0;
///
/// let interval = Duration::seconds(1);
///
/// let mut controller = Threshold::new(
///     threshold,
///     Input::default(),
///     Output::default(),
///     interval,
/// ).set_inverted();
///
/// controller.poll(Utc::now());
/// ```
///
#[derive(Debug)]
pub struct Threshold<I, O>
where
    I: Fn() -> String,
    O: FnMut(bool),
{
    name: Option<String>,
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
    /// Create a new controller with a specific time as the first read time
    ///
    /// This is the recommended API for instantiation.
    pub fn new(threshold: f32, input: Input<I>, output: Output<O>, interval: Duration) -> Threshold<I, O> {
        Self {
            name: None,
            threshold,
            input,
            output,
            schedule: Scheduler::new(),
            interval,
            inverted: false,
        }.schedule_next(None)
    }

    /// Create a new controller without scheduling the first read
    ///
    /// [`Threshold::schedule_next()`] must be called after this function.
    ///
    /// [`Threshold::new()`] is the recommended API for instantiation.
    ///
    /// This method is only useful for testing purposes.
    pub fn new_without_scheduled(threshold: f32, input: Input<I>, output: Output<O>, interval: Duration) -> Threshold<I, O> {
        Self {
            name: None,
            threshold,
            input,
            output,
            schedule: Scheduler::new(),
            interval,
            inverted: false,
        }
    }

    /// Builder method to set the controller to be inverted
    ///
    /// This means that the output will be activated when the input is below the threshold and
    /// deactivated when the input is above the threshold.
    ///
    /// # Example
    /// ```
    /// use chrono::Duration;
    /// use equilibrium::controllers::Threshold;
    /// use equilibrium::Input;
    /// use equilibrium::Output;
    ///
    /// let controller = Threshold::new(
    ///   5.0,
    ///   Input::default(),
    ///   Output::default(),
    ///  Duration::seconds(1)
    /// ).set_inverted();
    /// ```
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

    /// Builder method to schedule the next read for the specified time
    ///
    /// If no time is specified, the current time will be used.
    ///
    /// # Arguments
    /// * `time`: Time of the first event
    pub fn schedule_next<T>(mut self, time: T) -> Self
        where T: Into<Option<DateTime<Utc>>>
    {
        let time= time.into().unwrap_or_else(|| Utc::now());
        self.schedule.schedule_read(time + self.interval);
        self
    }
}

impl<I, O> Controller for Threshold<I, O>
    where
        I: Fn() -> String,
        O: FnMut(bool),
{
    fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    fn get_name(&self) -> Option<String> {
        self.name.clone()
    }

    /// Read the input and activate the output if the value is above the threshold
    ///
    /// The next read will be scheduled for the specified interval after the current time.
    fn poll(&mut self, time: DateTime<Utc>) -> Option<Message> {
        if let Some(event) = self.schedule.attempt_execution(time) {
            match event.get_action() {
                crate::types::Action::Read => {
                    // Read the input and handle the result
                    let msg = match self.above_threshold() {
                        true => {
                            self.handle_above_threshold();
                            "Above Threshold".to_string()
                        },
                        false => {
                            self.handle_below_threshold();
                            "Below Threshold".to_string()
                        }
                    };

                    // Schedule the next read
                    self.schedule.schedule_read(time + self.interval);

                    // prepare Message
                    let read_state = self.input.get_state().clone();
                    return Some(Message::new(
                        self.get_name().unwrap_or_default(),
                        msg,
                        time,
                        read_state,
                    ))
                }
                _ => panic!("Encountered unexpected action in threshold controller")
            }
        }
        None
    }
}

impl Default for Threshold<fn() -> String, fn(bool)> {
    fn default() -> Self {
        Self::new_without_scheduled(
            0.0,
            Input::default(),
            Output::default(),
            Duration::seconds(1)
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
        let threshold = 0.0;
        let interval = Duration::seconds(1);

        let input = Input::default();
        let output = Output::default();
        let controller = Threshold::new_without_scheduled(
            threshold,
            input,
            output,
            interval,
        );

        assert_eq!(controller.get_threshold(), 0.0);
        assert_eq!(controller.inverted, false);
        assert_eq!(controller.interval, Duration::seconds(1));
        assert!(!controller.schedule.has_future_events());
    }

    #[test]
    fn test_with_first() {
        let threshold = 0.0;
        let interval = Duration::seconds(1);

        let input = Input::default();
        let output = Output::default();
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
    fn test_set_inverted() {
        // check default
        let input = Input::default();
        let output = Output::default();
        let controller = Threshold::new_without_scheduled(
            0.0,
            input,
            output,
            Duration::seconds(1)
        );

        assert_eq!(controller.inverted, false);

        // check after setting
        let input = Input::default();
        let output = Output::default();
        let controller = Threshold::new_without_scheduled(
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
        let input = Input::default();
        let output = Output::default();
        let controller = Threshold::new_without_scheduled(
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
        let input = Input::default();
        let output = Output::default();
        let mut controller = Threshold::new_without_scheduled(
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
        let output = Output::default();
        let mut controller = Threshold::new_without_scheduled(
            5.0,
            input,
            output,
            Duration::seconds(1)
        );

        assert_eq!(controller.above_threshold(), false);

        // check when above threshold
        let input = Input::new(|| String::from("10.0"));
        let output = Output::default();
        let mut controller = Threshold::new_without_scheduled(
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
        let mut controller = Threshold::new_without_scheduled(
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

        let mut controller = Threshold::new_without_scheduled(
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

        let mut controller = Threshold::new_without_scheduled(
            5.0,
            input,
            output,
            Duration::seconds(1),
        ).schedule_next(time);

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
    fn test_get_set_name() {
        let mut controller = Threshold::default();

        assert_eq!(controller.get_name(), None);

        controller.set_name(String::from("test"));

        assert_eq!(controller.get_name(), Some(String::from("test")));
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

        let mut controller = Threshold::new_without_scheduled(
            5.0,
            input,
            output,
            Duration::seconds(1),
        )
            .set_inverted()
            .schedule_next(time);

        // check default state
        assert_eq!(external_output_state.lock().unwrap().clone(), false);

        // check before first read
        let message = controller.poll(time + Duration::milliseconds(500));
        assert_eq!(external_output_state.lock().unwrap().clone(), false);

        assert!(message.is_none());

        // check after first read when below threshold
        let message = controller.poll(time + Duration::seconds(1));
        assert_eq!(external_output_state.lock().unwrap().clone(), true);

        assert!(message.is_some());
        assert_eq!(message.as_ref().unwrap().get_read_state().unwrap(), "0.0");
        assert_eq!(message.as_ref().unwrap().get_content(), "Below Threshold");

        // check before second poll execution
        let message = controller.poll(time + Duration::milliseconds(1500));
        assert_eq!(external_output_state.lock().unwrap().clone(), true);

        assert!(message.is_none());

        // check after second read when above threshold
        let message = controller.poll(time + Duration::seconds(2));
        assert_eq!(external_output_state.lock().unwrap().clone(), false);

        assert!(message.is_some());
        assert_eq!(message.as_ref().unwrap().get_read_state().unwrap(), "10.0");
        assert_eq!(message.as_ref().unwrap().get_content(), "Above Threshold");

        // check after second read before third read
        let message = controller.poll(time + Duration::microseconds(2500));
        assert_eq!(external_output_state.lock().unwrap().clone(), false);

        assert!(message.is_none());

        // check after third read when below threshold
        let message = controller.poll(time + Duration::seconds(3));
        assert_eq!(external_output_state.lock().unwrap().clone(), true);

        assert!(message.is_some());
        assert_eq!(message.as_ref().unwrap().get_read_state().unwrap(), "0.0");
        assert_eq!(message.as_ref().unwrap().get_content(), "Below Threshold");
    }
}