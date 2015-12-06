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
use image::DynamicImage;

use std::cell::RefCell;

use command::prelude::*;
use datatypes::{CellData, Coords, MediaPosition};
use datatypes::Movement::Position;

pub struct Put(RefCell<Option<CellData>>);

impl Put {
    pub fn new_char(ch: char) -> Put {
        Put(RefCell::new(Some(CellData::Char(ch))))
    }
    pub fn new_extension(ch: char) -> Put {
        Put(RefCell::new(Some(CellData::ExtensionChar(ch))))
    }
    pub fn new_image(data: DynamicImage, pos: MediaPosition, w: u32, h: u32) -> Put {
        Put(RefCell::new(Some(CellData::Image {
            pos: pos,
            width: w,
            height: h,
            data: data
        })))
    }
}

impl Command for Put {

    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        if let Some(data) = self.0.borrow_mut().take() {
            terminal.write(data)
        }
        Ok(())
    }

    fn repr(&self) -> String {
        match *self.0.borrow() {
            Some(CellData::Char(c)) | Some(CellData::ExtensionChar(c))
                                            => c.to_string(),
            _                               => String::from("PUT"),
        }
    }

}

pub struct PutAt(RefCell<Option<CellData>>, Coords);

impl PutAt {

    pub fn new_image(data: DynamicImage, pos: MediaPosition, w: u32, h: u32, at: Coords) -> PutAt {
        PutAt(RefCell::new(Some(CellData::Image {
            pos: pos,
            width: w,
            height: h,
            data: data,
        })), at)
    }
}

impl Command for PutAt {

    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        if let Some(data) = self.0.borrow_mut().take() {
            let coords = terminal.cursor_position();
            terminal.move_cursor(Position(self.1));
            terminal.write(data);
            terminal.move_cursor(Position(coords));
        }
        Ok(())
    }

    fn repr(&self) -> String {
        String::from("PUT AT")
    }

}
