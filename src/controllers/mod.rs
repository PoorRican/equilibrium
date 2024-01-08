use chrono::{DateTime, Utc};

mod threshold;
mod bidirectional;
mod timed;

pub use threshold::Threshold;
pub use bidirectional::BidirectionalThreshold;
pub use timed::TimedOutput;

use crate::types::Message;

pub trait Controller {
    fn set_name(&mut self, name: String);
    fn get_name(&self) -> Option<String>;
    fn poll(&mut self, time: DateTime<Utc>) -> Option<Message>;
}