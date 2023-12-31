pub struct Output<F>
where F: FnMut(bool) {
    callback: F,
    state: bool,
}

impl<F> Output<F>
where F: FnMut(bool) {
    pub fn new(callback: F) -> Output<F> {
        Output {
            callback,
            state: false,
        }
    }

    pub fn activate(&mut self) {
        self.state = true;
        (self.callback)(true);
    }

    pub fn deactivate(&mut self) {
        self.state = false;
        (self.callback)(false);
    }

    pub fn get_state(&self) -> bool {
        self.state
    }
}