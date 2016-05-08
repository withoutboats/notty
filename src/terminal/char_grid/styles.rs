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
use datatypes::{Color, Style};
use datatypes::Style::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Styles {
    pub fg_color: Color,
    pub bg_color: Color,
    pub opacity: u8,
    pub underline: bool,
    pub double_underline: bool,
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub inverted: bool,
    pub blink: bool,
}

impl Default for Styles {
    fn default() -> Styles {
         Styles {
            fg_color:           Color::Default,
            bg_color:           Color::Default,
            opacity:            0xff,
            bold:               false,
            italic:             false,
            underline:          false,
            double_underline:   false,
            strikethrough:      false,
            inverted:           false,
            blink:              false,
        }
   }
}

impl Styles {

    pub fn new() -> Styles {
        Styles::default()
    }

    pub fn update(&mut self, style: Style) {
        match style {
            Underline(0)            => { self.underline = false; self.double_underline = false; }
            Underline(1)            => { self.underline = true;  self.double_underline = false; }
            Underline(2)            => { self.underline = false; self.double_underline = true;  }
            Underline(_)            => unreachable!(),
            Bold(flag)              => self.bold = flag,
            Italic(flag)            => self.italic = flag,
            Strikethrough(flag)     => self.strikethrough = flag,
            InvertColors(flag)      => self.inverted = flag,
            Blink(flag)             => self.blink = flag,
            Opacity(n)              => self.opacity = n,
            FgColor(color)          => self.fg_color = color,
            BgColor(color)          => self.bg_color = color,
        }
    }

}

#[cfg(test)]
mod tests {

    use datatypes::Color;
    use datatypes::Style::*;
    use super::*;

    #[test]
    fn styles_update() {
        let mut style = Styles::new();
        style.update(Bold(true));
        assert_eq!(style.bold, true);
        style.update(Italic(true));
        assert_eq!(style.italic, true);
        style.update(Underline(1));
        assert_eq!(style.underline, true);
        style.update(Underline(2));
        assert_eq!(style.double_underline, true);
        style.update(Strikethrough(true));
        assert_eq!(style.strikethrough, true);
        style.update(InvertColors(true));
        assert_eq!(style.inverted, true);
        style.update(Blink(true));
        assert_eq!(style.blink, true);

        style.update(FgColor(Color::True(0x10, 0x10, 0x10)));
        assert_eq!(style.fg_color, Color::True(0x10, 0x10, 0x10));
        style.update(BgColor(Color::True(0x10, 0x10, 0x10)));
        assert_eq!(style.bg_color, Color::True(0x10, 0x10, 0x10));
    }

}
