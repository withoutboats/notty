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
use std::rc::Rc;

use cfg::Config;
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

impl Styles {
    pub fn new(config: &Rc<Config>) -> Styles {
        Styles {
            fg_color:           config.fg_color,
            bg_color:           config.bg_color,
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
    pub fn update(&mut self, style: Style, config: &Rc<Config>) {
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
            FgColorCfg(Some(n))     => self.fg_color = config.colors[n as usize],
            FgColorCfg(None)        => self.fg_color = config.fg_color,
            BgColor(color)          => self.bg_color = color,
            BgColorCfg(Some(n))     => self.bg_color = config.colors[n as usize],
            BgColorCfg(None)        => self.bg_color = config.bg_color,
        }
    }
}

#[cfg(test)]
mod tests {

    use std::rc::Rc;

    use cfg::Config;
    use datatypes::Color;
    use datatypes::Style::*;
    use super::*;

    #[test]
    fn styles_update() {
        let config = Rc::new(Config::default());
        let mut style = Styles::new(&config);
        style.update(Bold(true), &config);
        assert_eq!(style.bold, true);
        style.update(Italic(true), &config);
        assert_eq!(style.italic, true);
        style.update(Underline(1), &config);
        assert_eq!(style.underline, true);
        style.update(Underline(2), &config);
        assert_eq!(style.double_underline, true);
        style.update(Strikethrough(true), &config);
        assert_eq!(style.strikethrough, true);
        style.update(InvertColors(true), &config);
        assert_eq!(style.inverted, true);
        style.update(Blink(true), &config);
        assert_eq!(style.blink, true);

        style.update(FgColor(Color(0x10, 0x10, 0x10)), &config);
        assert_eq!(style.fg_color, Color(0x10, 0x10, 0x10));
        style.update(BgColor(Color(0x10, 0x10, 0x10)), &config);
        assert_eq!(style.bg_color, Color(0x10, 0x10, 0x10));

        style.update(FgColorCfg(None), &config);
        assert_eq!(style.fg_color, config.fg_color);
        style.update(BgColorCfg(None), &config);
        assert_eq!(style.bg_color, config.bg_color);
    }

}
