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
use datatypes::Style;

#[derive(Copy, Clone)]
pub struct SetCursorStyle(pub Style);

impl Command for SetCursorStyle {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.set_cursor_style(self.0);
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("SET CURSOR STYLE")
    }
}

#[derive(Copy, Clone)]
pub struct DefaultCursorStyle;

impl Command for DefaultCursorStyle {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.reset_cursor_styles();
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("DEFAULT CURSOR STYLE")
    }
}
