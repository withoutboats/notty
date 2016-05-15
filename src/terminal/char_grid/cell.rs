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
use std::sync::Arc;

use mime::Mime;

use datatypes::{Coords, MediaPosition};
use terminal::Styles;

use self::CharData::*;

#[derive(Clone, PartialEq, Debug)]
pub struct CharCell {
    pub styles: Styles,
    pub content: CharData,
}

#[derive(Clone, PartialEq, Debug)]
pub enum CharData {
    Empty,
    Char(char),
    Grapheme(String),
    Extension(Coords),
    Image { 
        data: Arc<ImageData>,
        mime: Mime,
        pos: MediaPosition,
        width: u32,
        height: u32,
    }
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct ImageData {
    pub data: Vec<u8>,
    coords: Coords,
}

impl CharCell {

    pub fn new(styles: Styles) -> CharCell {
        CharCell {
            styles: styles,
            content: Empty,
        }
    }

    pub fn character(ch: char, styles: Styles) -> CharCell {
        CharCell {
            styles: styles,
            content: Char(ch)
        }
    }

    pub fn grapheme(grapheme: String, styles: Styles) -> CharCell {
        CharCell {
            styles: styles,
            content: Grapheme(grapheme)
        }
    }

    pub fn image(data: Vec<u8>,
                 coords: Coords,
                 mime: Mime,
                 pos: MediaPosition, 
                 width: u32,
                 height: u32,
                 styles: Styles) -> CharCell {
        CharCell {
            styles: styles,
            content: Image {
                data: Arc::new(ImageData {
                    data: data,
                    coords: coords,
                }),
                mime: mime,
                pos: pos,
                width: width,
                height: height
            }
        }
    }

    pub fn extension(coords: Coords, styles: Styles) -> CharCell {
        CharCell {
            styles: styles,
            content: Extension(coords),
        }
    }

    pub fn extend_by(&mut self, ext: char) -> bool {
        match self.content {
            Char(c)             => {
                let mut string = c.to_string();
                string.push(ext);
                self.content = Grapheme(string);
                true
            }
            Grapheme(ref mut s) => {
                s.push(ext);
                true
            }
            _                   => false
        }
    }

    pub fn repr(&self) -> String {
        match self.content {
            Char(c)         => c.to_string(),
            Grapheme(ref s) => s.clone(),
            Image { .. }    => String::from("IMG"),
            Empty           => String::new(),
            Extension(_)    => String::from("EXT"),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.content == Empty
    }

    pub fn is_char_extension(&self) -> bool {
        if let Extension(..) = self.content { true } else { false }
    }

}

impl Default for CharCell {
    fn default() -> CharCell {
        CharCell::new(Styles::new())
    }
}

impl ToString for CharCell {
    fn to_string(&self) -> String {
        match self.content {
            Char(c)         => c.to_string(),
            Grapheme(ref s) => s.clone(),
            _               => String::new()
        }
    }
}
