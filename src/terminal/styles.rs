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
use datatypes::{Color, ConfigStyle, Style, DEFAULT_CONFIG_STYLE};
use datatypes::Style::*;
use self::UseStyles::*;

pub const DEFAULT_STYLES: UseStyles = Config(DEFAULT_CONFIG_STYLE);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UseStyles {
    Custom(Styles),
    Config(ConfigStyle),
}

impl UseStyles {
    pub fn update(&mut self, style: Style) {
        *self = match (*self, style) {
            (_, Configured(style))      => Config(style),
            (Custom(styles), _)         => Custom(styles.update(style)),
            (Config(_), _)              => Custom(Styles::new().update(style)),
        }
    }
}

impl Default for UseStyles {
    fn default() -> UseStyles {
        DEFAULT_STYLES
    }
}

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

    pub fn update(self, style: Style) -> Styles {
        match style {
            Underline(0)        => Styles { underline: false, double_underline: false, ..self },
            Underline(1)        => Styles { underline: true, double_underline: false, ..self },
            Underline(2)        => Styles { underline: false, double_underline: true, ..self },
            Underline(_)        => unreachable!(),
            Bold(flag)          => Styles { bold: flag, ..self },
            Italic(flag)        => Styles { italic: flag, ..self },
            Strikethrough(flag) => Styles { strikethrough: flag, ..self },
            InvertColors(flag)  => Styles { inverted: flag, ..self },
            Blink(flag)         => Styles { blink: flag, ..self },
            Opacity(n)          => Styles { opacity: n, ..self },
            FgColor(color)      => Styles { fg_color: color, ..self },
            BgColor(color)      => Styles { bg_color: color, ..self },
            Configured(_)       => unreachable!(),
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
        let style = Styles::new();
        assert!(style.update(Bold(true)).bold);
        assert!(style.update(Italic(true)).italic);
        assert!(style.update(Underline(1)).underline);
        assert!(style.update(Underline(2)).double_underline);
        assert!(style.update(Strikethrough(true)).strikethrough);
        assert!(style.update(InvertColors(true)).inverted);
        assert!(style.update(Blink(true)).blink);
        let color = Color::True(0x10, 0x10, 0x10);
        assert_eq!(style.update(FgColor(color)).fg_color, color);
        assert_eq!(style.update(BgColor(color)).bg_color, color);
    }

}
