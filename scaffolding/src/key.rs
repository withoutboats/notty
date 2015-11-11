use std::cell::Cell;

use gdk::{EventKey, EventType};
use natty::datatypes::{InputEvent, Key};

pub fn translate(key: &EventKey, scroll: &Cell<usize>) -> Option<InputEvent> {
    if super_mode(key) {
        if key.keyval == 0xff52 {
            scroll.set(scroll.get() + 5);
            return None;
        } else if key.keyval == 0xff54 {
            scroll.set(scroll.get().saturating_sub(5));
            return None;
        }
    }
    let press = is_press(&key._type);
    scroll.set(0);
    Some(InputEvent::Key(match key.keyval {
        b @ 0x20...0x7e => Key::Char(press, b as u8 as char),
        0xff08          => Key::Char(press, '\x08'),
        0xff09          => Key::Char(press, '\x09'),
        0xff0a          => Key::Char(press, '\x0a'),
        0xff0d          => Key::Char(press, '\x0d'),
        0xff14          => Key::ScrollLock(press),
        0xff1b          => Key::Char(press, '\x1b'),
        0xff50          => Key::Home(press),
        0xff51          => Key::Left(press),
        0xff52          => Key::Up(press),
        0xff53          => Key::Right(press),
        0xff54          => Key::Down(press),
        0xff55          => Key::PageUp(press),
        0xff56          => Key::PageDown(press),
        0xff57          => Key::End(press),
        0xffe1          => Key::ShiftLeft(press),
        0xffe2          => Key::ShiftRight(press),
        0xffe3          => Key::CtrlLeft(press),
        0xffe4          => Key::CtrlRight(press),
        0xffe5          => Key::CapsLock(press),
        0xffe7 | 0xffeb => Key::MetaLeft(press),
        0xffe8 | 0xff67 => Key::MetaRight(press),
        0xffe9          => Key::AltLeft(press),
        0xffea          => Key::AltRight(press),
        0xffff          => Key::Char(press, '\x7f'),
        x               => { panic!("Key press: {:x}", x) }
    }))
}

fn is_press(event_type: &EventType) -> bool {
    match *event_type {
        EventType::KeyPress     => true,
        EventType::KeyRelease   => false,
        _                       => unreachable!(),
    }
}

fn super_mode(key: &EventKey) -> bool {
    key.state.bits() & 0o100 != 0
}
