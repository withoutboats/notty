use std::borrow::Cow;

use command::prelude::*;
use datatypes::{Key, Coords, Code};

pub struct StaticResponse(pub &'static str);

impl Command for StaticResponse {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.send_input(Key::Cmd(Cow::Borrowed(self.0)), true)
    }
    fn repr(&self) -> String {
        String::from("RESPOND ") + self.0
    }
}

pub struct ReportPosition(pub Code);

impl Command for ReportPosition {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        let Coords { x, y } = terminal.cursor_position();
        let cmd = match self.0 {
            Code::ANSI  => Cow::Owned(format!("\x1b[{};{}R", y, x)),
            _           => unimplemented!(),
        };
        terminal.send_input(Key::Cmd(cmd), true)
    }
    fn repr(&self) -> String {
        String::from("REPORT POSITION")
    }
}
