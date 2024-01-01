use chrono::{DateTime, Duration, NaiveTime, Utc};
use crate::controllers::Controller;
use crate::output::Output;

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
///
pub struct TimedOutput<F>
where F: FnMut(bool) {
    output: Output<F>,
    start_time: NaiveTime,
    duration: Duration,
}

impl<F> TimedOutput<F>
where F: FnMut(bool) {
    pub fn new(output: Output<F>, start_time: NaiveTime, duration: Duration) -> Self {
        Self {
            output,
            start_time,
            duration,
        }
    }
}

impl<F> Controller for TimedOutput<F>
where F: FnMut(bool) {
    fn poll(&mut self, time: DateTime<Utc>) {
        let time = time.naive_utc().time();
        let end_time = self.start_time + self.duration;
        if time >= self.start_time && time < end_time {
            self.output.activate();
        } else {
            self.output.deactivate();
        }
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