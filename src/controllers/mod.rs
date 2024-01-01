use chrono::{DateTime, Utc};

mod threshold;
mod bidirectional;
mod timed;

pub trait Controller {
    fn poll(&mut self, time: DateTime<Utc>);
}