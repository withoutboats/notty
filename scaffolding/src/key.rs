use std::cell::Cell;

use gdk::EventKey;
use natty::{Command, KeyPress, KeyRelease};
use natty::datatypes::Key;

pub trait FromEvent {
    fn from_event(&EventKey, &Cell<usize>) -> Option<Box<Command>>;
}

impl FromEvent for KeyPress {
    fn from_event(key: &EventKey, scroll: &Cell<usize>) -> Option<Box<Command>> {
        if super_mode(key) {
            if key.keyval == 0xff52 {
                scroll.set(scroll.get() + 5);
                return None;
            } else if key.keyval == 0xff54 {
                scroll.set(scroll.get().saturating_sub(5));
                return None;
            }
        }
        scroll.set(0);
        Some(Box::new(KeyPress(match key.keyval {
            b @ 0x20...0x7e => Key::Char(b as u8 as char),
            0xff08          => Key::Char('\x08'),
            0xff09          => Key::Char('\x09'),
            0xff0a          => Key::Char('\x0a'),
            0xff0d          => Key::Char('\x0d'),
            0xff14          => Key::ScrollLock,
            0xff1b          => Key::Char('\x1b'),
            0xff50          => Key::Home,
            0xff51          => Key::Left,
            0xff52          => Key::Up,
            0xff53          => Key::Right,
            0xff54          => Key::Down,
            0xff55          => Key::PageUp,
            0xff56          => Key::PageDown,
            0xff57          => Key::End,
            0xffe1          => Key::ShiftLeft,
            0xffe2          => Key::ShiftRight,
            0xffe3          => Key::CtrlLeft,
            0xffe4          => Key::CtrlRight,
            0xffe5          => Key::CapsLock,
            0xffe7 | 0xffeb => Key::MetaLeft,
            0xffe8 | 0xff67 => Key::MetaRight,
            0xffe9          => Key::AltLeft,
            0xffea          => Key::AltRight,
            0xffff          => Key::Char('\x7f'),
            x               => { panic!("Key press: {:x}", x) }
        })) as Box<Command>)
    }
}

impl FromEvent for KeyRelease {
    fn from_event(key: &EventKey, scroll: &Cell<usize>) -> Option<Box<Command>> {
        if super_mode(key) && (key.keyval == 0xff52 || key.keyval == 0xff54) {
            return None;
        }
        scroll.set(0);
        Some(Box::new(KeyRelease(match key.keyval {
            b @ 0x20...0x7e => Key::Char(b as u8 as char),
            0xff08          => Key::Char('\x08'),
            0xff09          => Key::Char('\x09'),
            0xff0a          => Key::Char('\x0a'),
            0xff0d          => Key::Char('\x0d'),
            0xff14          => Key::ScrollLock,
            0xff1b          => Key::Char('\x1b'),
            0xff50          => Key::Home,
            0xff51          => Key::Left,
            0xff52          => Key::Up,
            0xff53          => Key::Right,
            0xff54          => Key::Down,
            0xff55          => Key::PageUp,
            0xff56          => Key::PageDown,
            0xff57          => Key::End,
            0xffe1          => Key::ShiftLeft,
            0xffe2          => Key::ShiftRight,
            0xffe3          => Key::CtrlLeft,
            0xffe4          => Key::CtrlRight,
            0xffe5          => Key::CapsLock,
            0xffe7 | 0xffeb => Key::MetaLeft,
            0xffe8 | 0xff67 => Key::MetaRight,
            0xffe9          => Key::AltLeft,
            0xffea          => Key::AltRight,
            0xffff          => Key::Char('\x7f'),
            x               => { panic!("Key press: {:x}", x) }
        })) as Box<Command>)
    }
}

fn super_mode(key: &EventKey) -> bool {
    key.state.bits() & 0o100 == 0o100
}
