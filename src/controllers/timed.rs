use chrono::{DateTime, Duration, NaiveTime, Utc};
use crate::controllers::Controller;
use crate::output::Output;
use crate::types::Message;

/// Simple controller that turns on an output at a specific time and turns it off after a duration.
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
/// ```
/// use chrono::{Duration, NaiveTime, Utc};
/// use equilibrium::controllers::{Controller, TimedOutput};
/// use equilibrium::output::Output;
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
}

impl<F> TimedOutput<F>
where F: FnMut(bool) {
    pub fn new(output: Output<F>, start_time: NaiveTime, duration: Duration) -> Self {
        Self {
            name: None,
            output,
            start_time,
            duration,
        }
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
        let naive_time = time.naive_utc().time();
        let end_time = self.start_time + self.duration;
        let msg = if naive_time >= self.start_time && naive_time < end_time {
            self.output.activate();
            "Activating".to_string()
        } else {
            self.output.deactivate();
            "Deactivating".to_string()
        };
        Some(Message::new(
            self.get_name().unwrap_or_default(),
            msg,
            time))
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
        let time = NaiveTime::from_hms_opt(5, 0, 0).unwrap();
        let duration = Duration::hours(12);
        let mut output = TimedOutput::new(
            Output::default(),
            time,
            duration,
        );

        assert_eq!(output.output.get_state(), None);

        let time = Utc.with_ymd_and_hms(2021, 1, 1, 4, 59, 59).unwrap();
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