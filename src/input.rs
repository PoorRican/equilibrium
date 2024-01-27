/// Encapsulates an input device
///
/// An input device is characterized by a physical device that can be read from.
/// The low-level code to perform the read is encapsulated in the `Input` struct
/// by providing a callback function that returns a `String`.
///
/// The `Input` struct also maintains the state of the input device, which is
/// updated every time the input is read.
///
/// # Example
/// ```
/// use equilibrium::Input;
///
/// let input = Input::new(|| {
///      // low-level code would go here
///      String::from("1.0")
/// });
/// ```
#[derive(Debug)]
pub struct Input<F>
where F: Fn() -> String {
    callback: F,
    state: Option<String>,
}

impl<F> Input<F>
where F: Fn() -> String {
    /// Create a new `Input` instance
    ///
    /// # Arguments
    /// * `callback` - Low-level code that returns input as a `String`
    pub fn new(callback: F) -> Input<F> {
        Input {
            callback,
            state: None,
        }
    }

    /// Read the input
    ///
    /// The callback function is executed and the internal state is executed.
    pub fn read(&mut self) -> String {
        let state = (self.callback)();
        self.state = Some(state.clone());
        state
    }

    /// Get the current state of the input
    ///
    /// The state is treated as a cache of the last read value and gets updated
    /// every time the input is read.
    pub fn get_state(&self) -> &Option<String> {
        &self.state
    }
}

impl Default for Input<fn() -> String> {
    /// The default callback function returns an empty `String`
    fn default() -> Self {
        Self::new(|| String::new())
    }
}


#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_new() {
        let input = super::Input::new(|| String::from("test"));

        assert_eq!(input.get_state(), &None);
    }

    #[test]
    fn test_read() {
        let mut input = super::Input::new(|| String::from("test"));

        assert_eq!(input.get_state(), &None);

        // Read the input
        let state = input.read();
        assert_eq!(state, String::from("test"));
        assert_eq!(input.get_state(), &Some(String::from("test")));
    }

    /// An example that shows how to get a dynamic input in tests
    #[test]
    fn test_read_twice() {
        let state_sequence =
            Arc::new(
                Mutex::new(
                    VecDeque::from([
                        "test1".to_string(),
                        "test2".to_string()
            ])));
        let mut input = super::Input::new(|| {
            let mut state_sequence = state_sequence.lock().unwrap();
            state_sequence.pop_front().unwrap()
        });

        assert_eq!(input.get_state(), &None);

        // Read the input
        let state = input.read();
        assert_eq!(state, String::from("test1"));
        assert_eq!(input.get_state(), &Some(String::from("test1")));

        // Read the input again
        let state = input.read();
        assert_eq!(state, String::from("test2"));
        assert_eq!(input.get_state(), &Some(String::from("test2")));
    }
}