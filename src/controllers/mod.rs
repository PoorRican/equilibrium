use chrono::{DateTime, Utc};

mod threshold;
mod bidirectional;
mod timed;

pub use threshold::Threshold;
pub use bidirectional::BidirectionalThreshold;
pub use timed::TimedOutput;

pub trait Controller {
    fn set_name(&mut self, name: String);
    fn get_name(&self) -> Option<String>;
    fn poll(&mut self, time: DateTime<Utc>);
}