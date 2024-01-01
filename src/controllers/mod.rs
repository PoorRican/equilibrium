use chrono::{DateTime, Utc};

mod threshold;
mod bidirectional;

pub trait Controller {
    fn poll(&mut self, time: DateTime<Utc>);
}