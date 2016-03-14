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

use notty_encoding::cmds::{
    PushPanel, PopPanel,
    SplitPanel, UnsplitPanel,
    SwitchActivePanel,
    SetInputMode
};

use command::prelude::*;
use datatypes::InputSettings;

impl Command for PushPanel {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.push(self.0);
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("PUSH BUFFER")
    }
}

impl Command for PopPanel {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.pop(self.0);
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("POP BUFFER")
    }
}

impl Command for SplitPanel {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.split(self.save, self.kind, self.rule, self.split_tag, self.l_tag, self.r_tag);
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("SPLIT BUFFER")
    }
}

impl Command for UnsplitPanel {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.unsplit(self.save, self.unsplit_tag);
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("UNSPLIT BUFFER")
    }
}

impl Command for SwitchActivePanel {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.switch(self.0);
        Ok(())
    }
    fn repr(&self) -> String {
        format!("SWITCH TO PANEL {}", self.0)
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
