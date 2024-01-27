//! Controller templates with defined scheduling, and a variety of input/output configurations.
//!
//! Each controller internally handles scheduling and performs reads and activates/deactivates outputs
//! accordingly. Most controller implementations operate on intervals (e.g.: 1 second intervals), however
//! it is possible to create a controller instance that schedules events in any way. For example, the
//! [`TimedOutput`] controller actuates its output at a certain time every day for a set duration.
//!
//! Each controller defines a [`poll()`](Controller::poll) function, which returns a [`Option<Message>`].
//! When a controller is polled, it evaluates whether an [`Action`](crate::types::Action) should be performed or not.
//! If an [`Action`](crate::types::Action) is performed, a [`Message`] is returned for logging.
//!
//! The controllers are fully documented and contain potential use-cases, examples, and more detailed information.
use chrono::{DateTime, Utc};

mod threshold;
mod bidirectional;
mod timed;

pub use threshold::Threshold;
pub use bidirectional::BidirectionalThreshold;
pub use timed::TimedOutput;

use crate::types::Message;

/// A trait that represents a named device that can be polled for events
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