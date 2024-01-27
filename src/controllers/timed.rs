use chrono::{DateTime, Duration, NaiveTime, Timelike, Utc};
use crate::controllers::Controller;
use crate::output::Output;
use crate::scheduler::Scheduler;
use crate::types::Message;

/// Simple controller that turns on an output at a specific time and turns it off after a duration has passed.
///
/// This is used to repeat the same action every day at the same time.
///
/// # Potential Use Cases
/// * Controlling grow lights
/// * Regularly turning on an O2 pump for a fish tank or bioreactor
/// * Regularly dumping a sedimentation filter
/// * Controlling a feed motor for fish feed
///
/// # Example
/// In this example, the output will be actuated at 5:00AM and deactivated after 8 hours (1:00PM)
/// ```
/// use chrono::{Duration, NaiveTime, Utc};
/// use equilibrium::controllers::{Controller, TimedOutput};
/// use equilibrium::Output;
///
/// let time = NaiveTime::from_hms_opt(5, 0, 0).unwrap();
/// let duration = Duration::hours(8);
/// let mut output = TimedOutput::new(
///   Output::default(),
///   time,
///   duration,
/// );
///
/// output.poll(Utc::now());
/// ```
#[derive(Debug)]
pub struct TimedOutput<F>
where F: FnMut(bool) {
    name: Option<String>,
    output: Output<F>,
    start_time: NaiveTime,
    duration: Duration,
    scheduler: Scheduler,
}

impl<F> TimedOutput<F>
where F: FnMut(bool) {
    /// Create a new timed output
    ///
    /// This does not schedule the first event and [`TimedOutput::schedule_first`]
    /// should be used to schedule the first event. It is recommended to use the [`TimedOutput::with_first`]
    /// method instead.
    pub fn new(output: Output<F>, start_time: NaiveTime, duration: Duration) -> Self {
        Self {
            name: None,
            output,
            start_time,
            duration,
            scheduler: Scheduler::new(),
        }
    }

    /// Create a new timed output and schedule the first event
    ///
    /// This is the recommended API for instantiating new [`TimeOutput`]s.
    pub fn with_first(output: Output<F>, start_time: NaiveTime, duration: Duration) -> Self {
        Self {
            name: None,
            output,
            start_time,
            duration,
            scheduler: Scheduler::new(),
        }.schedule_first(None)
    }


    /// Schedule the first event
    fn schedule_first<T>(mut self, time: T) -> Self
        where T: Into<Option<DateTime<Utc>>>
    {
        self.schedule_on(time);

        self
    }

    /// Determine the next time the output should be activated
    ///
    /// The next time that the output will be activated will be at the time specified by `start_time`,
    /// if the given time is between `start_time` and `start_time + duration`, the output will
    /// not be activated.
    ///
    /// # Arguments
    /// * `time` - The time to check for events that should be executed. If `None`, the current time will be used.
    fn schedule_on<T>(&mut self, time: T)
        where T: Into<Option<DateTime<Utc>>>
    {
        let mut time= time.into().unwrap_or_else(|| Utc::now());
        let current_time = time.naive_utc().time();

        // calculate the next time the output should be activated
        time = time.with_hour(self.start_time.hour()).unwrap();
        time = time.with_minute(self.start_time.minute()).unwrap();
        time = time.with_second(self.start_time.second()).unwrap();
        time = time.with_nanosecond(0).unwrap();

        let start_time = if current_time < self.start_time {
            time
        } else {
            time + Duration::days(1)
        };

        self.scheduler.schedule_on(start_time);
    }

    /// Determine the next time the output should be deactivated
    ///
    /// # Arguments
    /// * `time` - The time to check for events that should be executed. If `None`, the current time will be used.
    fn schedule_off<T>(&mut self, time: T)
        where T: Into<Option<DateTime<Utc>>>
    {
        let mut time= time.into().unwrap_or_else(|| Utc::now());

        // calculate the next time the output should be deactivated
        time = time.with_hour(self.start_time.hour()).unwrap();
        time = time.with_minute(self.start_time.minute()).unwrap();
        time = time.with_second(self.start_time.second()).unwrap();
        time = time.with_nanosecond(0).unwrap();

        let end_time = time + self.duration;
        self.scheduler.schedule_off(end_time);
    }
}

impl<F> Controller for TimedOutput<F>
where F: FnMut(bool) {
    fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    fn get_name(&self) -> Option<String> {
        self.name.clone()
    }

    fn poll(&mut self, time: DateTime<Utc>) -> Option<Message> {
        if let Some(event) = self.scheduler.attempt_execution(time) {
            let msg = match event.get_action() {
                crate::types::Action::On => {
                    self.output.activate();
                    self.schedule_off(time);
                    "Activated"
                },
                crate::types::Action::Off => {
                    self.output.deactivate();
                    self.schedule_on(time);
                    "Deactivated"
                },
                _ => {
                    panic!("Invalid action for timed output")
                }
            };
            return Some(Message::new(
                self.get_name().unwrap_or_default(),
                String::from(msg),
                time,
                None,
            ))
        }
        None
    }
}

impl Default for TimedOutput<fn(bool)> {
    fn default() -> Self {
        Self::new(Output::default(), NaiveTime::from_hms_opt(0, 0, 0).unwrap(), Duration::seconds(0))
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use super::*;

    #[test]
    fn test_new() {
        let time = NaiveTime::from_hms_opt(5, 0, 0).unwrap();
        let duration = Duration::hours(8);
        let output = TimedOutput::new(
            Output::default(),
            time,
            duration,
        );

        assert_eq!(output.output.get_state(), None);
    }

    #[test]
    fn test_get_set_name() {
        let mut controller = TimedOutput::default();

        assert_eq!(controller.get_name(), None);

        controller.set_name(String::from("test"));
        assert_eq!(controller.get_name(), Some(String::from("test")));
    }

    #[test]
    fn test_poll() {
        // time to use for polling
        let time = Utc.with_ymd_and_hms(2021, 1, 1, 4, 59, 59).unwrap();

        let start_time = NaiveTime::from_hms_opt(5, 0, 0).unwrap();
        let duration = Duration::hours(12);
        let mut output = TimedOutput::new(
            Output::default(),
            start_time,
            duration,
        ).schedule_first(time);

        assert_eq!(output.output.get_state(), None);

        // force change output state to false
        output.output.deactivate();

        // begin polling
        output.poll(time);
        assert_eq!(output.output.get_state().unwrap(), false);

        let time = time + Duration::seconds(1);
        output.poll(time);
        assert_eq!(output.output.get_state().unwrap(), true);

        let time = time + Duration::hours(6);
        output.poll(time);
        assert_eq!(output.output.get_state().unwrap(), true);

        let time = time + Duration::hours(6) - Duration::seconds(1);
        output.poll(time);
        assert_eq!(output.output.get_state().unwrap(), true);

        let time = time + Duration::seconds(1);
        output.poll(time);
        assert_eq!(output.output.get_state().unwrap(), false);
    }

}