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
use mime::Mime;

use command::prelude::*;
use datatypes::{Coords, MediaPosition};
use datatypes::Movement::Position;
use terminal::{CharData, WideChar, CharExtender, Image};

pub struct Put<T: CharData>(T);

impl Put<char> {
    pub fn new_char(ch: char) -> Put<char> {
        Put(ch)
    }
}

impl Put<WideChar> {
    pub fn new_wide_char(ch: char, width: u32) -> Put<WideChar> {
        Put(WideChar::new(ch, width))
    }
}

impl Put<CharExtender> {
    pub fn new_extender(ch: char) -> Put<CharExtender> {
        Put(CharExtender::new(ch))
    }
}

impl Put<Image> {
    pub fn new_image(data: Vec<u8>, mime: Mime, pos: MediaPosition, w: u32, h: u32) -> Put<Image> {
        Put(Image::new(data, mime, pos, w, h))
    }
}

impl<T: CharData> Command for Put<T> {

    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        if let Some(grid) = terminal.grid_mut() {
            grid.write(&self.0);
        }
        Ok(())
    }

    #[cfg(any(test, debug_assertions))]
    fn repr(&self) -> String {
        self.0.repr()
    }

}

pub struct PutAt<T: CharData>(T, Coords);

impl PutAt<Image> {

    pub fn new_image(data: Vec<u8>, mime: Mime, pos: MediaPosition, w: u32, h: u32, at: Coords)
            -> PutAt<Image> {
        PutAt(Image::new(data, mime, pos, w, h), at)
    }
}

impl<T: CharData> Command for PutAt<T> {

    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        if let Some(grid) = terminal.grid_mut() {
            let coords = grid.cursor().position();
            grid.move_cursor(Position(self.1));
            grid.write(&self.0);
            grid.move_cursor(Position(coords));
        }
        Ok(())
    }

    #[cfg(any(test, debug_assertions))]
    fn repr(&self) -> String {
        String::from("PUT AT")
    }

}
