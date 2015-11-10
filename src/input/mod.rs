use std::io::{self, Write};

use datatypes::{InputEvent, InputMode, Key};
use datatypes::InputMode::*;
use datatypes::Key::*;

mod modifiers;

use self::modifiers::Modifiers;

pub struct Input<W: Write> {
    tty: W,
    mode: InputMode,
    modifiers: Modifiers
}

impl<W> Input<W> where W: Write {

    pub fn new(tty: W) -> Input<W> {
        Input {
            tty: tty,
            mode: InputMode::Ansi,
            modifiers: Modifiers::new(),
        }
    }

    pub fn process(&mut self, event: InputEvent) -> io::Result<()> {
        match event {
            InputEvent::Key(key)    => {
                if self.mode == InputMode::Extended {
                    self.write(key)
                } else {
                    match key {
                        Key::ShiftLeft(flag) | Key::ShiftRight(flag) => {
                            self.modifiers.shift = flag;
                            Ok(())
                        }
                        Key::CtrlLeft(flag) | Key::CtrlRight(flag) => {
                            self.modifiers.ctrl = flag;
                            Ok(())
                        }
                        Key::AltLeft(flag) | Key::AltRight(flag) => {
                            self.modifiers.alt = flag;
                            Ok(())
                        }
                        Key::CapsLock(true) => {
                            self.modifiers.caps = !self.modifiers.caps;
                            Ok(())
                        }
                        Key::CapsLock(false) => Ok(()),
                        _   => self.write(key),
                    }
                }
            }
            InputEvent::Mode(mode)  => {
                self.set_mode(mode);
                Ok(())
            }
        }
    }

    fn write(&mut self, key: Key) -> io::Result<()> {
        match self.mode {
            Ansi        => match self.ansi_encode(key, false) {
                Some(code)  => self.tty.write_all(code.as_bytes()),
                None        => Ok(())
            },
            Application => match self.ansi_encode(key, true) {
                Some(code)  => self.tty.write_all(code.as_bytes()),
                None        => Ok(())
            },
            Extended    => unimplemented!()
        }
    }

    fn set_mode(&mut self, mode: InputMode) {
        self.mode = mode;
    }

    fn ansi_encode(&self, key: Key, application: bool) -> Option<String> {
        match key {
            Char(true, c)                       => char_key(self.modifiers, c),
            Cmd(s)                              => Some(s.into_owned()),
            Up(true)                            => term_key(self.modifiers, 'A', application),
            Down(true)                          => term_key(self.modifiers, 'B', application),
            Left(true)                          => term_key(self.modifiers, 'D', application),
            Right(true)                         => term_key(self.modifiers, 'C', application),
            ShiftLeft(_)
                | ShiftRight(_)
                | CtrlLeft(_)
                | CtrlRight(_)
                | AltLeft(_)
                | AltRight(_)
                | CapsLock(_)                   => unreachable!(),
            MetaLeft(true) | MetaRight(true)    => None,
            PageUp(true)                        => tilde_key(self.modifiers, '5'),
            PageDown(true)                      => tilde_key(self.modifiers, '6'),
            Home(true)                          => term_key(self.modifiers, 'H', true),
            End(true)                           => term_key(self.modifiers, 'F', true),
            Insert(true)                        => tilde_key(self.modifiers, '2'),
            Delete(true)                        => tilde_key(self.modifiers, '3'),
            NumLock(_)                          => unimplemented!(),
            ScrollLock(_)                       => unimplemented!(),
            Function(..)                        => unimplemented!(),
            _                                   => None,
        }
    }

}

fn char_key(modifiers: Modifiers, c: char) -> Option<String> {
    match (modifiers.ctrl, modifiers.alt) {
        (false,  false) => Some(c.to_string()),
        (true,   false) => match c {
            c @ '\x40'...'\x7f' => Some((((c as u8) & 0b00011111) as char).to_string()),
            _                   => None,
        },
        (false,  true)  => Some(format!("\x1b{}", c)),
        (true,   true)  => match c {
            c @ '\x40'...'\x7f' => Some(format!("\x1b{}", ((c as u8) & 0b00011111 ) as char)),
            _                   => None,
        }
    }
}

fn term_key(modifiers: Modifiers, term: char, application: bool) -> Option<String> {
    match modifiers.triplet() {
        (false, false, false) if application    => Some(format!("\x1bO{}", term)),
        (false, false, false)                   => Some(format!("\x1b[{}", term)),
        (true,  false, false)                   => Some(format!("\x1b[1;2{}", term)),
        (false, false, true)                    => Some(format!("\x1b[1;3{}", term)),
        (true,  false, true)                    => Some(format!("\x1b[1;4{}", term)),
        (false, true,  false)                   => Some(format!("\x1b[1;5{}", term)),
        (true,  true,  false)                   => Some(format!("\x1b[1;6{}", term)),
        (false, true,  true)                    => Some(format!("\x1b[1;7{}", term)),
        (true,  true,  true)                    => Some(format!("\x1b[1;8{}", term)),
    }
}

fn tilde_key(modifiers: Modifiers, init: char) -> Option<String> {
    match modifiers.triplet() {
        (false, false, false)           => Some(format!("\x1b[{}~", init)),
        (true,  false, false)           => Some(format!("\x1b[{};2~", init)),
        (false, false, true)            => Some(format!("\x1b[{};3~", init)),
        (true,  false, true)            => Some(format!("\x1b[{};4~", init)),
        (false, true,  false)           => Some(format!("\x1b[{};5~", init)),
        (true,  true,  false)           => Some(format!("\x1b[{};6~", init)),
        (false, true,  true)            => Some(format!("\x1b[{};7~", init)),
        (true,  true,  true)            => Some(format!("\x1b[{};8~", init)),
    }
}
