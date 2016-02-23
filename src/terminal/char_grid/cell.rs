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

use cfg::Config;
use datatypes::{Coords, MediaPosition};
use terminal::Styles;

use self::CharCell::*;

#[derive(Eq, PartialEq, Hash)]
pub struct ImageData {
    pub data: Vec<u8>,
    coords: Coords,
}

#[derive(Clone)]
pub enum CharCell {
    Empty(Styles),
    Char(char, Styles),
    Grapheme(String, Styles),
    Image(Arc<ImageData>, Mime, MediaPosition, (u32, u32), Styles),
    Extension(Coords, Styles),
}

impl CharCell {

    pub fn new(config: &Config) -> CharCell {
        Empty(Styles::new(&config))
    }

    pub fn character(ch: char, style: Styles) -> CharCell {
        Char(ch, style)
    }

    pub fn grapheme(grapheme: String, style: Styles) -> CharCell {
       Grapheme(grapheme, style)
    }

    pub fn image(data: Vec<u8>,
                 coords: Coords,
                 mime: Mime,
                 pos: MediaPosition, 
                 width: u32,
                 height: u32,
                 style: Styles) -> CharCell {
        Image(Arc::new(ImageData {
            data: data,
            coords: coords,
        }), mime, pos, (width, height), style)
    }

    pub fn extend_by(&mut self, ext: char) -> bool {
        match *self {
            Char(c, style)          => {
                let mut string = c.to_string();
                string.push(ext);
                *self = Grapheme(string, style);
                true
            }
            Grapheme(ref mut s, _)  => {
                s.push(ext);
                true
            }
            _                       => {
                false
            }
        }
    }

    pub fn repr(&self) -> String {
        match *self {
            Char(c, _)          => c.to_string(),
            Grapheme(ref s, _)  => s.clone(),
            Image(..)           => String::from("IMG"),
            Empty(_)            => String::new(),
            Extension(..)       => String::from("EXT"),
        }
    }

    pub fn style(&self) -> &Styles {
        match *self {
            Char(_, ref style)
                | Grapheme(_, ref style)
                | Empty(ref style)
                | Image(_, _, _, _, ref style)
                | Extension(_, ref style)
                => style
        }
    }

    pub fn style_mut(&mut self) -> &mut Styles {
        match *self {
            Char(_, ref mut style)
                | Grapheme(_, ref mut style)
                | Empty(ref mut style)
                | Image(_, _, _, _, ref mut style) 
                | Extension(_, ref mut style)
                => style
        }
    }

    pub fn empty(&mut self) {
        let style = *self.style();
        *self = Empty(style);
    }

    pub fn is_empty(&self) -> bool {
        if let Empty(_) = *self { true } else { false }
    }

    pub fn is_char_extension(&self) -> bool {
        if let Extension(..) = *self { true } else { false }
    }

}
