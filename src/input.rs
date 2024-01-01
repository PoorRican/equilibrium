#[derive(Debug)]
pub struct Input<F>
where F: Fn() -> String {
    callback: F,
    state: Option<String>,
}

impl<F> Input<F>
where F: Fn() -> String {
    pub fn new(callback: F) -> Input<F> {
        Input {
            callback,
            state: None,
        }
    }

    pub fn read(&mut self) -> String {
        let state = (self.callback)();
        self.state = Some(state.clone());
        state
    }

    pub fn get_state(&self) -> &Option<String> {
        &self.state
    }
}

impl Default for Input<fn() -> String> {
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

    /// An example that shows how to get a dynamic input
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