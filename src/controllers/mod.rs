use chrono::{DateTime, Utc};

mod threshold;

pub trait Controller {
    fn poll(&mut self, time: DateTime<Utc>);
}