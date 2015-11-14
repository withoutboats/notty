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
use datatypes::{Area, Style};

#[derive(Copy, Clone)]
pub struct SetStyleInArea(pub Area, pub Style);

impl Command for SetStyleInArea {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.set_style_in_area(self.0, self.1);
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("SET STYLE IN AREA")
    }
}

#[derive(Copy, Clone)]
pub struct DefaultStyleInArea(pub Area);

impl Command for DefaultStyleInArea {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.reset_styles_in_area(self.0);
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("DEFAULT STYLE IN AREA")
    }
}
