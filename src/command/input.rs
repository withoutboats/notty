use command::prelude::*;
use datatypes::Key;

pub struct KeyPress(pub Key);

impl Command for KeyPress {
    fn apply(&self, terminal: &mut Terminal) {
        terminal.send_input(self.0.clone(), true);
    }
    fn repr(&self) -> String {
        String::from("KEY PRESS")
    }
}

pub struct KeyRelease(pub Key);

impl Command for KeyRelease {
    fn apply(&self, terminal: &mut Terminal) {
        terminal.send_input(self.0.clone(), false);
    }
    fn repr(&self) -> String {
        String::from("KEY RELEASE")
    }
}
