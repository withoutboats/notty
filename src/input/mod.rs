use std::io::{self, Write};
use std::sync::mpsc::Receiver;

use datatypes::{Key, InputEvent, InputMode, Modifiers};

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
            modifiers: Modifiers::new()
        }
    }

    pub fn run(&mut self, rx: Receiver<InputEvent>) -> io::Result<()> {
        for item in rx {
            match item {
                InputEvent::Key(key)     => if self.mode == InputMode::Extended {
                    try!(self.write(key));
                } else {
                    match key {
                        Key::ShiftLeft(b) | Key::ShiftRight(b)  => self.modifiers.shift = b,
                        Key::CtrlLeft(b) | Key::CtrlRight(b)    => self.modifiers.ctrl = b,
                        Key::AltLeft(b) | Key::AltRight(b)      => self.modifiers.alt = b,
                        Key::CapsLock(true)                     => self.modifiers.caps = true,
                        Key::CapsLock(false)                    => (),
                        _                                       => try!(self.write(key)),
                    }
                },
                InputEvent::Mode(mode)   => self.set_mode(mode),
            }
        }
        unreachable!()
    }

    fn write(&mut self, key: Key) -> io::Result<()> {
        if let Some(string) = key.as_code(self.mode, self.modifiers) {
            self.tty.write_all(string.as_bytes())
        } else { Ok(()) }
    }

    fn set_mode(&mut self, mode: InputMode) {
        self.mode = mode;
    }

}
