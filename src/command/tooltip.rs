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

use command::prelude::*;
use datatypes::Coords;

pub struct AddToolTip(pub Coords, pub RefCell<Option<String>>);

impl Command for AddToolTip {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        if let Some(string) = self.1.borrow_mut().take() {
            terminal.add_tooltip(self.0, string);
        }
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("ADD TOOL TIP")
    }
}

pub struct RemoveToolTip(pub Coords);

impl Command for RemoveToolTip {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.remove_tooltip(self.0);
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("REMOVE TOOL TIP")
    }
}

pub struct AddDropDown {
    pub coords: Coords,
    pub options: RefCell<Option<Vec<String>>>,
}

impl Command for AddDropDown {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        if let Some(options) = self.options.borrow_mut().take() {
            terminal.add_drop_down(self.coords, options);
        }
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("ADD TOOL TIP - DROP DOWN MENU")
    }
}
