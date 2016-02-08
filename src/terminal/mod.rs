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
use std::io::{self, Write};
use std::ops::{Deref, DerefMut};

mod char_grid;
mod screen;
mod input;

use Command;
use datatypes::{InputSettings, Key};

pub use self::char_grid::{CharCell, CharGrid, Cursor, Grid, Styles, Tooltip, ImageData};
pub use self::input::Tty;
pub use self::screen::{Screen, ScreenIter, SaveGrid, SplitKind, ResizeRule};

use self::input::Input;

pub struct Terminal {
    title: String,
    screen: Screen,
    tty: Input,
}

impl Terminal {

    pub fn new<W: Tty + Send + 'static>(width: u32, height: u32, tty: W) -> Terminal {
        Terminal {
            title: String::new(),
            screen: Screen::new(width, height),
            tty: Input::new(tty),
        }
    }

    pub fn apply(&mut self, cmd: &Command) -> io::Result<()> {
        cmd.inner.apply(self)
    }

    pub fn send_input(&mut self, key: Key, press: bool) -> io::Result<()> {
        if let Some(cmd) = try!(match key {
            Key::DownArrow | Key::UpArrow | Key::Enter if press => {
                let cursor = self.cursor_position();
                match match self.tooltip_at_mut(cursor) {
                    Some(tooltip @ &mut Tooltip::Menu { .. })   => tooltip.interact(&key),
                    _                                           => Err(true)
                } {
                    Ok(n)       => self.tty.write(Key::MenuSelection(n), true),
                    Err(true)   => self.tty.write(key, press),
                    Err(false)  => Ok(None),
                }
            }
            _           => self.tty.write(key, press),
        }) {
            try!(cmd.inner.apply(self));
        }
        Ok(())
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn set_input_mode(&mut self, mode: InputSettings) {
        self.tty.set_mode(mode);
    }

    pub fn bell(&mut self) {
        println!("BELL");
    }

    pub fn set_winsize(&mut self, cols: u32, rows: u32, rule: ResizeRule) -> io::Result<()> {
        self.resize(Some(cols), Some(rows), rule);
        self.tty.set_winsize(cols, rows)
    }

}

impl Deref for Terminal {
    type Target = Screen;
    fn deref(&self) -> &Screen {
        &self.screen
    }
}

impl DerefMut for Terminal {
    fn deref_mut(&mut self) -> &mut Screen {
        &mut self.screen
    }
}
