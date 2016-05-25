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

use Command;
use datatypes::{InputSettings, Key};

mod buffer;
mod ansi;
mod line_echo;
mod modifiers;
mod notty;
mod screen_echo;

use self::buffer::InputBuffer;
use self::line_echo::LineEcho;
use self::modifiers::Modifiers;
use self::notty::Extended;
use self::screen_echo::ScreenEcho;
use self::InputMode::*;

pub trait Tty: Write {
    fn set_winsize(&mut self, u16, u16) -> io::Result<()>;
}

pub struct Input {
    tty: Box<Tty + Send>,
    mode: InputMode,
    paste_mode: PasteMode,
    modifiers: Modifiers,
}

impl Input {

    pub fn new<W: Tty + Send + 'static>(tty: W) -> Input {
        Input {
            tty: Box::new(tty),
            mode: Ansi(false),
            paste_mode: PasteMode::Silent,
            modifiers: Modifiers::new(),
        }
    }

    pub fn set_mode(&mut self, mode: InputSettings) {
        match mode {
            InputSettings::Ansi(flag)                   =>
                self.mode = Ansi(flag),
            InputSettings::Notty(_)                     =>
                self.mode = ExtendedRaw(Extended),
            InputSettings::LineBufferEcho(echo, buffer) =>
                self.mode = ExtendedLineBuffer(LineEcho::new(echo), InputBuffer::new(buffer)),
            InputSettings::ScreenEcho(settings)         =>
                self.mode = ExtendedScreen(ScreenEcho::new(settings), Extended),
            InputSettings::BracketedPasteMode(_)        => (),
        };
        self.paste_mode = match mode {
            InputSettings::BracketedPasteMode(true)     => PasteMode::Bracketed,
            InputSettings::BracketedPasteMode(false)    => PasteMode::Silent,
            _                                           => self.paste_mode,

        };
    }

    pub fn set_winsize(&mut self, width: u32, height: u32) -> io::Result<()> {
        self.tty.set_winsize(width as u16, height as u16)
    }

    pub fn write(&mut self, key: Key, press: bool) -> io::Result<Option<Command>> {
        if key.is_modifier() { self.modifiers.apply(&key, press); }
        let key = if self.modifiers.ctrl() { key.ctrl_modify() } else { key };
        self.mode.write(key, press, &mut self.tty, self.modifiers)
    }

    pub fn paste(&mut self, data: &str) -> io::Result<Option<Command>> {
        self.mode.paste(data, &mut self.tty, self.paste_mode)
    }

}

enum InputMode {
    Ansi(bool),
    ExtendedRaw(Extended),
    ExtendedLineBuffer(LineEcho, InputBuffer),
    ExtendedScreen(ScreenEcho, Extended),
}

#[derive(Copy, Clone)]
enum PasteMode {
    Silent,
    Bracketed,
}

impl InputMode {

    fn write(&mut self, key: Key, press: bool, tty: &mut Write, modifiers: Modifiers)
            -> io::Result<Option<Command>> {
        match *self {
            Ansi(app_mode) if press && !key.is_modifier() => {
                if let Some(data) = ansi::encode(&key, app_mode, modifiers) {
                    tty.write_all(data.as_bytes()).and(Ok(None))
                } else { Ok(None) }
            }
            ExtendedRaw(notty)                  => {
                let data = notty.encode(&key, press, modifiers);
                tty.write_all(data.as_bytes()).and(Ok(None))
            }
            ExtendedLineBuffer(ref mut echo, ref mut buffer) => {
                if let Some(data) = buffer.write(&key, echo.settings) {
                    try!(tty.write_all(data.as_bytes()))
                }
                if press { Ok(echo.echo(key)) } else { Ok(None) }
            }
            ExtendedScreen(ref mut echo, notty) => {
                let data = notty.encode(&key, press, modifiers);
                try!(tty.write_all(data.as_bytes()));
                if press { Ok(echo.echo(key)) } else { Ok(None) }
            }
            _                                   => Ok(None)
        }
    }

    fn paste(&self, data: &str, tty: &mut Write, paste_mode: PasteMode)
            -> io::Result<Option<Command>> {
        match (self, paste_mode) {
            (&Ansi(_), PasteMode::Bracketed)    =>
                write!(tty, "\x1b[200~{}\x1b[201~", data).and(Ok(None)),
            (&Ansi(_), PasteMode::Silent)       =>
                write!(tty, "{}", data).and(Ok(None)),
            _               => unimplemented!(),
        }
    }

}
