use std::borrow::Cow;

use command::prelude::*;
use datatypes::{Key, Coords, Code};

pub struct StaticResponse(pub &'static str);

impl Command for StaticResponse {
    fn apply(&self, _: &mut Screen, tx: &Sender<InputEvent>) {
        tx.send(InputEvent::Key(Key::Cmd(Cow::Borrowed(self.0))));
    }
    fn repr(&self) -> String {
        String::from("RESPOND ") + self.0
    }
}

pub struct ReportPosition(pub Code);

impl Command for ReportPosition {
    fn apply(&self, screen: &mut Screen, tx: &Sender<InputEvent>) {
        let Coords { x, y } = screen.cursor_position();
        let cmd = match self.0 {
            Code::ANSI  => Cow::Owned(format!("\x1b[{};{}R", y, x)),
            _           => unimplemented!(),
        };
        tx.send(InputEvent::Key(Key::Cmd(cmd)));
    }
    fn repr(&self) -> String {
        String::from("REPORT POSITION")
    }
}
