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
use command::prelude::*;
use datatypes::Key;

pub struct KeyPress(pub Key);

impl Command for KeyPress {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.send_input(self.0.clone(), true)
    }
    fn repr(&self) -> String {
        String::from("KEY PRESS")
    }
}

pub struct KeyRelease(pub Key);

impl Command for KeyRelease {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.send_input(self.0.clone(), false)
    }
    fn repr(&self) -> String {
        String::from("KEY RELEASE")
    }
}
