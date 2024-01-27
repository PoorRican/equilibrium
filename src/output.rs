/// Encapsulates an output device.
///
/// An output device is characterized by a physical device that can be activated or deactivated.
/// At the moment, only binary outputs are supported, but this may change in the future. The low-level
/// code to perform the activation/deactivation is encapsulated in the `Output` struct by providing
/// a callback function that accepts a `bool` argument.
///
/// The `Output` struct also maintains the state of the output device, which is updated every time
/// the output is activated or deactivated.
///
/// # Example
/// ```
/// use equilibrium::Output;
///
/// let output = Output::new(|state| {
///     // low-level code would go here
///     println!("Output state: {}", state);
/// });
/// ```
#[derive(Debug)]
pub struct Output<F>
where F: FnMut(bool) {
    callback: F,
    state: Option<bool>,
}

impl<F> Output<F>
where F: FnMut(bool) {
    /// Create a new `Output` instance
    ///
    /// # Arguments
    /// * `callback` - Low-level code that accepts a `bool` argument
    pub fn new(callback: F) -> Output<F> {
        Output {
            callback,
            state: None,
        }
    }

    /// Activate the output
    pub fn activate(&mut self) {
        self.state = Some(true);
        (self.callback)(true);
    }

    /// Deactivate the output
    pub fn deactivate(&mut self) {
        self.state = Some(false);
        (self.callback)(false);
    }

    /// Get the current state of the output
    ///
    /// The state is treated as a cache of the last activated/deactivated value and gets updated
    /// every time the output is activated or deactivated.
    pub fn get_state(&self) -> Option<bool> {
        self.state
    }
}

impl Default for Output<fn(bool)> {
    /// The default callback function does nothing
    fn default() -> Self {
        Self::new(|_| {})
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_new() {
        let output = super::Output::new(|_| {});

        assert_eq!(output.get_state(), None);
    }

    #[test]
    fn test_activate() {
        let external_state = Arc::new(Mutex::new(false));
        let mut output = super::Output::new(|state| {
            let mut external_state = external_state.lock().unwrap();
            *external_state = state;
        });

        // check default state
        assert_eq!(output.get_state(), None);

        output.activate();
        assert_eq!(output.get_state().unwrap(), true);
        assert_eq!(external_state.lock().unwrap().clone(), true);
    }

    #[test]
    fn test_deactivate() {

        let external_state = Arc::new(Mutex::new(true));
        let mut output = super::Output::new(|state| {
            let mut external_state = external_state.lock().unwrap();
            *external_state = state;
        });

        // check default state
        assert_eq!(output.get_state(), None);

        output.deactivate();

        assert_eq!(external_state.lock().unwrap().clone(), false);
        assert_eq!(output.get_state().unwrap(), false);
    }
}