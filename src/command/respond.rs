//  notty is a new kind of terminal emulator.
//  Copyright (C) 2015 without boats
//  
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//  
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU Affero General Public License for more details.
//  
//  You should have received a copy of the GNU Affero General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
use std::borrow::Cow;

use command::prelude::*;
use datatypes::{Key, Coords, Code};

pub struct StaticResponse(pub &'static str);

impl Command for StaticResponse {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.send_input(Key::Cmd(Cow::Borrowed(self.0)), true)
    }

    #[cfg(any(test, debug_assertions))]
    fn repr(&self) -> String {
        String::from("RESPOND ") + self.0
    }
}

pub struct ReportPosition(pub Code);

impl Command for ReportPosition {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        let Coords { x, y } = terminal.cursor().position();
        let cmd = match self.0 {
            Code::ANSI  => Cow::Owned(format!("\x1b[{};{}R", y, x)),
            _           => unimplemented!(),
        };
        terminal.send_input(Key::Cmd(cmd), true)
    }

    #[cfg(any(test, debug_assertions))]
    fn repr(&self) -> String {
        String::from("REPORT POSITION")
    }
}
