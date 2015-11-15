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

use datatypes::{InputMode, Key};
use datatypes::InputMode::*;

mod modifiers;
mod ansi;
mod notty;


use self::modifiers::Modifiers;

pub struct Input {
    tty: Box<Write>,
    mode: InputMode,
    modifiers: Modifiers
}

impl Input {

    pub fn new<W: Write + 'static>(tty: W) -> Input {
        Input {
            tty: Box::new(tty),
            mode: InputMode::Ansi(false),
            modifiers: Modifiers::new(),
        }
    }

    pub fn set_mode(&mut self, mode: InputMode) {
        self.mode = mode;
    }

    pub fn write(&mut self, key: Key, press: bool) -> io::Result<()> {
        match self.mode {
            InputMode::Ansi(_) if key.is_modifier()     => {
                self.modifiers.apply(&key, press);
                Ok(())
            }
            InputMode::Ansi(app_mode) if press          => {
                match ansi::encode(key, app_mode, self.modifiers) {
                    Some(code)  => self.tty.write_all(code.as_bytes()),
                    None        => Ok(()),
                }
            }
            InputMode::Ansi(_)                          => Ok(()),
            InputMode::Notty(flags)                     => {
                if key.is_modifier() {
                    self.modifiers.apply(&key, press);
                }
                self.tty.write_all(notty::encode(key, press, flags, self.modifiers).as_bytes())
            }
        }
    }

}
