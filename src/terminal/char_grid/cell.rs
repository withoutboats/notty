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
use datatypes::Coords;
use terminal::{UseStyles, DEFAULT_STYLES};
use terminal::image::Image as ImageData;
use terminal::interfaces::{Cell, Styleable, WriteableCell};

use self::CellData::*;

pub const EMPTY_CELL: CharCell = CharCell {
    styles: DEFAULT_STYLES,
    content: CellData::Empty,
};

#[derive(Clone, PartialEq, Debug)]
pub struct CharCell {
    styles: UseStyles,
    content: CellData,
}

impl CharCell {
    pub fn content(&self) -> &CellData {
        &self.content
    }
}

impl Default for CharCell {
    fn default() -> CharCell {
        CharCell {
            content: Empty,
            styles: DEFAULT_STYLES,
        }
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

impl Styleable for CharCell {
    fn styles(&self) -> &UseStyles {
        &self.styles
    }

    fn styles_mut(&mut self) -> &mut UseStyles {
        &mut self.styles
    }
}

impl Cell for CharCell {
    fn is_extension(&self) -> bool {
        self.source().is_some()
    }

    fn erase(&mut self) {
        self.content = CellData::Empty;
        self.styles = DEFAULT_STYLES;
    }
}

impl WriteableCell for CharCell {
    fn write(&mut self, data: CellData, styles: UseStyles) {
        self.content = data;
        self.styles = styles;
    }

    fn extend(&mut self, extension: char, styles: UseStyles) {
        if let CellData::Char(c) = self.content {
            self.content = CellData::Grapheme(format!("{}{}", c, extension));
            self.styles = styles;
        } else if let CellData::Grapheme(ref mut s) = self.content {
            s.push(extension);
            self.styles = styles;
        }
    }

    fn is_extendable(&self) -> bool {
        match self.content {
            CellData::Char(_) | CellData::Grapheme(_)   => true,
            _                                           => false,
        }
    }

    fn source(&self) -> Option<Coords> {
        match self.content {
            CellData::Extension(coords) => Some(coords),
            _                           => None,
        }
    }
}

#[cfg(any(test, debug_assertions))]
impl CharCell {
    pub fn repr(&self) -> String {
        match self.content {
            Char(c)         => c.to_string(),
            Grapheme(ref s) => s.clone(),
            Image(_)        => String::from("IMG"),
            Empty           => String::new(),
            Extension(_)    => String::from("EXT"),
        }
    }

}

#[derive(Clone, PartialEq, Debug)]
pub enum CellData {
    Empty,
    Char(char),
    Grapheme(String),
    Extension(Coords),
    Image(ImageData),
}

#[cfg(test)]
mod tests {
    use datatypes::Coords;
    use terminal::interfaces::*;
    use terminal::UseStyles;
    use super::*;

    fn character() -> CharCell {
        CharCell {
            content: CellData::Char('a'),
            styles: UseStyles::default(),
        }
    }

    fn extension() -> CharCell {
        CharCell {
            content: CellData::Extension(Coords { x: 0, y: 0 }),
            styles: UseStyles::default(),
        }
    }

    #[test]
    fn is_extension() {
        assert!(!character().is_extension());
        assert!(!CharCell::default().is_extension());
        assert!(extension().is_extension());
    }

    #[test]
    fn erase() {
        let mut cell = character();
        cell.erase();
        assert_eq!(cell, CharCell::default());
    }
}
