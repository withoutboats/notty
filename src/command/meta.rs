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
use std::cell::RefCell;

use notty_encoding::cmds::{PushBuffer, PopBuffer, SetInputMode};

use command::prelude::*;
use datatypes::InputSettings;

impl Command for PushBuffer {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.push_buffer(false, self.0);
        Ok(())
    }
    fn repr(&self) -> String {
        match self.0 {
            true    => String::from("PUSH BUFFER SCROLLING"),
            false   => String::from("PUSH BUFFER STATIC"),
        }
    }
}

impl Command for PopBuffer {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.pop_buffer();
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("POP BUFFER")
    }
}

pub struct SetTitle(pub RefCell<Option<String>>);

impl Command for SetTitle {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        if let Some(title) = self.0.borrow_mut().take() {
            terminal.set_title(title);
        }
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("SET TITLE")
    }
}

impl Command for SetInputMode {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.set_input_mode(self.0);
        Ok(())
    }
    fn repr(&self) -> String {
        match self.0 {
            InputSettings::Ansi(false)          => String::from("SET INPUTMODE ANSI"),
            InputSettings::Ansi(true)           => String::from("SET INPUTMODE APPLICATION"),
            InputSettings::Notty(_)             => String::from("SET INPUTMODE EXTENDED"),
            InputSettings::LineEcho(_)          => String::from("SET INPUTMODE LINE ECHO"),
            InputSettings::LineBufferEcho(_, _) => String::from("SET INPUTMODE LINEBUFFER ECHO"), 
            InputSettings::ScreenEcho(_)        => String::from("SET INPUTMODE SCREEN ECHO"),
        }
    }
}

#[derive(Default, Copy, Clone)]
pub struct Bell;

impl Command for Bell {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.bell();
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("BELL")
    }
}
