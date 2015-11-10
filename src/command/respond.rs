use std::borrow::Cow;

use command::prelude::*;
use datatypes::{Key, Coords, Code};

pub struct StaticResponse(pub &'static str);

impl Command for StaticResponse {
    fn apply(&self, _: &mut Screen, input: &mut FnMut(InputEvent)) {
        input(InputEvent::Key(Key::Cmd(Cow::Borrowed(self.0))));
    }
    fn repr(&self) -> String {
        String::from("RESPOND ") + self.0
    }
}

pub struct ReportPosition(pub Code);

impl Command for ReportPosition {
    fn apply(&self, screen: &mut Screen, input: &mut FnMut(InputEvent)) {
        let Coords { x, y } = screen.cursor_position();
        let cmd = match self.0 {
            Code::ANSI  => Cow::Owned(format!("\x1b[{};{}R", y, x)),
            _           => unimplemented!(),
        };
        input(InputEvent::Key(Key::Cmd(cmd)));
    }
    fn repr(&self) -> String {
        String::from("REPORT POSITION")
    }
}
