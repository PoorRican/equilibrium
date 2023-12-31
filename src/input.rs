pub struct Input<F>
where F: FnOnce() -> String {
    callback: F,
    state: String,
}

impl<F> Input<F>
where F: FnOnce() -> String {
    pub fn new(callback: F) -> Input<F> {
        Input {
            callback,
            state: String::new(),
        }
    }

    pub fn read(&mut self) -> String {
        self.state = (self.callback)();
        self.state.clone()
    }

    pub fn get_state(&self) -> &String {
        &self.state
    }
}