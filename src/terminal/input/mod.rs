use std::io::{self, Write};

use datatypes::{InputMode, Key};
use datatypes::InputMode::*;
use datatypes::Key::*;

mod modifiers;

use self::modifiers::Modifiers;

/// The `Input` struct processes `InputEvent`s and manages sending data from the terminal to the
/// controlling process.
pub struct Input {
    tty: Box<Write>,
    mode: InputMode,
    modifiers: Modifiers
}

impl Input {

    /// Create a new input processor by wraping a writeable interface to the tty (or whatever
    /// system you are using to connect the terminal to the stdin of the controlling process).
    ///
    /// The input processor defaults to ANSI compatibility mode.
    pub fn new<W: Write + 'static>(tty: W) -> Input {
        Input {
            tty: Box::new(tty),
            mode: InputMode::Ansi,
            modifiers: Modifiers::new(),
        }
    }

    pub fn process(&mut self, key: Key, press: bool) -> io::Result<()> {
        if self.mode == InputMode::Extended {
            self.write(key)
        } else {
            match key {
                Key::Cmd(s) => self.tty.write_all(s.as_bytes()),
                Key::ShiftLeft | Key::ShiftRight => {
                    self.modifiers.shift = press;
                    Ok(())
                }
                Key::CtrlLeft | Key::CtrlRight => {
                    self.modifiers.ctrl = press;
                    Ok(())
                }
                Key::AltLeft | Key::AltRight => {
                    self.modifiers.alt = press;
                    Ok(())
                }
                Key::CapsLock if press => {
                    self.modifiers.caps = !self.modifiers.caps;
                    Ok(())
                }
                _  if press => self.write(key),
                _           => Ok(())
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

    pub fn set_mode(&mut self, mode: InputMode) {
        self.mode = mode;
    }

    fn ansi_encode(&self, key: Key, application: bool) -> Option<String> {
        match key {
            Char(c)                 => char_key(self.modifiers, c),
            Up                      => term_key(self.modifiers, 'A', application),
            Down                    => term_key(self.modifiers, 'B', application),
            Left                    => term_key(self.modifiers, 'D', application),
            Right                   => term_key(self.modifiers, 'C', application),
            MetaLeft | MetaRight    => None,
            PageUp                  => tilde_key(self.modifiers, '5'),
            PageDown                => tilde_key(self.modifiers, '6'),
            Home                    => term_key(self.modifiers, 'H', true),
            End                     => term_key(self.modifiers, 'F', true),
            Insert                  => tilde_key(self.modifiers, '2'),
            Delete                  => tilde_key(self.modifiers, '3'),
            NumLock                 => unimplemented!(),
            ScrollLock              => unimplemented!(),
            Function(_)             => unimplemented!(),
            ShiftLeft
                | ShiftRight
                | CtrlLeft
                | CtrlRight
                | AltLeft
                | AltRight
                | CapsLock
                | Cmd(_)            => unreachable!(),
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
