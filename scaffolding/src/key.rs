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

use gdk::EventKey;
use notty::{Command, KeyPress, KeyRelease};
use notty::datatypes::Key;

pub trait FromEvent {
    fn from_event(&EventKey) -> Option<Box<Command>>;
}

impl FromEvent for KeyPress {
    fn from_event(key: &EventKey) -> Option<Box<Command>> {
        Some(Box::new(KeyPress(keyval(key.keyval))) as Box<Command>)
    }
}

impl FromEvent for KeyRelease {
    fn from_event(key: &EventKey) -> Option<Box<Command>> {
        if super_mode(key) && (key.keyval == 0xff52 || key.keyval == 0xff54) {
            return None;
        }
        Some(Box::new(KeyRelease(keyval(key.keyval))) as Box<Command>)
    }
}

fn super_mode(key: &EventKey) -> bool {
    key.state.bits() & 0o100 == 0o100
}

fn keyval(n: u32) -> Key {
    match n {
        b @ 0x20...0x7e => Key::Char(b as u8 as char),
        0xff08          => Key::Backspace,
        0xff09          => Key::Char('\x09'),
        0xff0a | 0xff0d => Key::Enter,
        0xff14          => Key::ScrollLock,
        0xff1b          => Key::Char('\x1b'),
        0xff50          => Key::Home,
        0xff51          => Key::LeftArrow,
        0xff52          => Key::UpArrow,
        0xff53          => Key::RightArrow,
        0xff54          => Key::DownArrow,
        0xff55          => Key::PageUp,
        0xff56          => Key::PageDown,
        0xff57          => Key::End,
        0xffe1          => Key::ShiftLeft,
        0xffe2          => Key::ShiftRight,
        0xffe3          => Key::CtrlLeft,
        0xffe4          => Key::CtrlRight,
        0xffe5          => Key::CapsLock,
        0xffe7 | 0xffeb => Key::Meta,
        0xffe8 | 0xff67 => Key::Menu,
        0xffe9          => Key::AltLeft,
        0xffea          => Key::AltGr,
        0xffff          => Key::Delete,
        x               => { panic!("Key press: {:x}", x) }
    }
}
