pub struct Output<F>
where F: FnMut(bool) {
    callback: F,
    state: Option<bool>,
}

impl<F> Output<F>
where F: FnMut(bool) {
    pub fn new(callback: F) -> Output<F> {
        Output {
            callback,
            state: None,
        }
    }

    pub fn activate(&mut self) {
        self.state = Some(true);
        (self.callback)(true);
    }

    pub fn deactivate(&mut self) {
        self.state = Some(false);
        (self.callback)(false);
    }

    pub fn get_state(&self) -> Option<bool> {
        self.state
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