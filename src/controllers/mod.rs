use chrono::{DateTime, Utc};

mod threshold;
mod bidirectional;
mod timed;

pub use threshold::Threshold;
pub use bidirectional::BidirectionalThreshold;
pub use timed::TimedOutput;

use crate::types::Message;

/// A `Controller` trait represents a named device that can be polled for events
///
/// There are no restrictions on the inputs or outputs of a controller, but
/// the controller must be able to be polled for events. Each instance of a
/// controller is responsible for responding to the poll request and performing
/// any necessary actions at the appropriate time.
///
/// Instances are not required to be named, however, the name is used for
/// logging purposes and will be contained in the `Message` returned by the
/// `poll` method.
pub trait Controller {
    /// Set the name of the controller
    fn set_name(&mut self, name: String);

    /// Get the name of the controller
    fn get_name(&self) -> Option<String>;

    /// Poll the controller for events
    ///
    /// The controller should return a `Message` if an event has occurred
    fn poll(&mut self, time: DateTime<Utc>) -> Option<Message>;
}