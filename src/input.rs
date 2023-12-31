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


#[cfg(test)]
mod tests {
    #[test]
    fn test_new() {
        let mut input = super::Input::new(|| String::from("test"));

        assert_eq!(input.get_state(), &None);

        // Read the input
        let state = input.read();
        assert_eq!(state, String::from("test"));
        assert_eq!(input.get_state(), &Some(String::from("test")));
    }
}