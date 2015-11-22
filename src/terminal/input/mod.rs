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

use command::Command;
use datatypes::{BufferSettings, EchoSettings, InputMode, Key};
use datatypes::InputMode::*;

mod buffer;
mod ansi;
mod echo;
mod modifiers;
mod notty;

use self::buffer::InputBuffer;
use self::modifiers::Modifiers;

pub struct Input {
    tty: Box<Write>,
    mode: InputMode,
    modifiers: Modifiers,
    buffer: InputBuffer,
    echo_set: Option<EchoSettings>,
    buffer_set: Option<BufferSettings>,
}

impl Input {

    pub fn new<W: Write + 'static>(tty: W) -> Input {
        Input {
            tty: Box::new(tty),
            mode: InputMode::Ansi(false),
            modifiers: Modifiers::new(),
            buffer: InputBuffer::default(),
            echo_set: None,
            buffer_set: None,
        }
    }

    pub fn set_mode(&mut self, mode: InputMode) {
        self.mode = mode;
    }

    pub fn set_echo(&mut self, echo: Option<EchoSettings>) {
        self.echo_set = echo;
    }

    pub fn set_buffer(&mut self, buffer: Option<BufferSettings>) {
        self.buffer_set = buffer;
    }

    pub fn write(&mut self, mut key: Key, press: bool) -> io::Result<Option<Box<Command>>> {
        if key.is_modifier() {
            self.modifiers.apply(&key, press);
        }
        if self.modifiers.ctrl() {
            key = key.ctrl_modify();
        }
        match (self.buffer_set, self.echo_set) {
            (Some(buffer), Some(echo)) if press => {
               if let Some(data) = self.buffer.write(&key, buffer, echo) {
                   try!(self.tty.write_all(data.as_bytes()));
               }
            }
            (None, _)   => try!(self.send(&key, press)),
            _           => ()
        }
        match self.echo_set {
            Some(set) if press  => {
                Ok(echo::encode(key, set.lerase as char, set.lnext as char, set.werase as char))
            }
            _                   => Ok(None),
        }
    }

    fn send(&mut self, key: &Key, press: bool) -> io::Result<()> {
        match self.mode {
            InputMode::Ansi(_) if key.is_modifier()     => {
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
                self.tty.write_all(notty::encode(key, press, flags, self.modifiers).as_bytes())
            }
        }
    }

}
