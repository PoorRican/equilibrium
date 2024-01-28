use chrono::{Duration, Utc};
use tokio::time::sleep;
use crate::{ControllerGroup, Emitter};

/// A wrapper around a [`ControllerGroup`] that runs the group at a specified interval
///
/// It has a loop that runs forever and polls the controllers. Any messages that are returned
/// are sent to an optional [`Emitter`] for logging.
///
/// An `interval` defines how often the group is polled. This must be low enough to ensure that
/// the controllers are polled often enough to meet their requirements. The loop will sleep for
/// 100ms between polls to avoid busy-looping, however, the [`Runtime::run`] method is very
/// greedy and will consume a substantial amount of CPU to ensure that the controllers are polled
/// as accurately as possible.
pub struct Runtime {
    emitter: Option<Emitter>,
    group: ControllerGroup,
    interval: Duration,
}

impl Runtime {
    /// Create a new runtime
    ///
    /// The default runtime does not have an emitter attached
    ///
    /// # Arguments
    /// * `group` - The controller group to run
    /// * `interval` - Operating interval
    ///
    /// # Returns
    /// A new runtime
    ///
    /// # Example
    /// ```
    /// use equilibrium::{Runtime, ControllerGroup, Input, Output};
    /// use chrono::Duration;
    /// use equilibrium::controllers::Threshold;
    ///
    ///
    /// let group = ControllerGroup::new()
    ///         .add_controller(Threshold::new(
    ///             70.0,
    ///             Input::default(),
    ///            Output::default(),
    ///           Duration::minutes(5)))
    ///         .add_controller(Threshold::new(
    ///             40.0,
    ///             Input::default(),
    ///             Output::default(),
    ///             Duration::minutes(1)));
    ///
    /// let runtime = Runtime::new(group, Duration::seconds(1));
    /// ```
    pub fn new(group: ControllerGroup, interval: Duration) -> Self {
        Self {
            emitter: None,
            group,
            interval,
        }
    }

    /// Builder method to add an emitter to the runtime
    ///
    /// # Arguments
    /// * `url` - The url to build the emitter with
    ///
    /// # Returns
    /// The runtime with the emitter attached
    ///
    /// # Example
    /// ```
    /// use equilibrium::{Emitter, Runtime, ControllerGroup};
    ///
    /// let runtime = Runtime::new(
    ///     ControllerGroup::new(),
    ///     chrono::Duration::seconds(1)
    /// ).build_emitter("http://localhost:8000");
    /// ```
    pub fn build_emitter<S>(mut self, url: S) -> Self
        where S: Into<String>
    {
        let emitter = Emitter::new(url);

        self.emitter = Some(emitter);
        self
    }

    /// Returns true if an emitter has been built
    pub fn has_emitter(&self) -> bool {
        self.emitter.is_some()
    }

    /// Execute the runtime
    ///
    /// This method will run forever and should be called from a tokio runtime
    pub async fn run(&mut self) {
        let mut next_execution_time = Utc::now() + self.interval;
        loop {
            let now = Utc::now();

            if now >= next_execution_time {
                // poll the group and get messages
                let messages = self.group.poll(now);

                if !messages.is_empty() {
                    if let Some(emitter) = &self.emitter {
                        emitter.emit(messages).await.unwrap();
                    }
                }

                // update the next execution time
                next_execution_time = now + self.interval;
            }

            // sleep for 100ms to avoid busy-looping
            sleep(std::time::Duration::milliseconds(100)).await
        }
    }
}