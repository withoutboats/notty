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

use gdk::{EventKey, EventType, CONTROL_MASK};
use notty::datatypes::{Direction, Key};
use notty::Command;

pub enum KeyEvent {
    Command(Command),
    Scroll(Direction),
    Copy,
    Paste,
    Ignore,
}

impl KeyEvent {
    pub fn new(event: &EventKey) -> KeyEvent {
        let ctor: fn(Key) -> Command = match event.get_event_type() {
            EventType::KeyPress     => Command::key_press,
            EventType::KeyRelease   => Command::key_release,
            _                       => unreachable!()
        };
        match (event.get_keyval(), shift_ctrl(event)) {
            // Shift+Ctrl+C
            (0x43, true) | (0x63, true) => KeyEvent::Copy,
            // Shift+Ctrl+V
            (0x56, true) | (0x76, true) => KeyEvent::Paste,
            // Shift+Ctrl+Up Arrow
            (0xff52, true)              => KeyEvent::Scroll(Direction::Up),
            // Shift+Ctrl+Down Arrow
            (0xff54, true)              => KeyEvent::Scroll(Direction::Down),
            // Shift+Ctrl+Left Arrow
            (0xff51, true)              => KeyEvent::Scroll(Direction::Left),
            // Shift+Ctrl+Right Arrow
            (0xff53, true)              => KeyEvent::Scroll(Direction::Right),
            (_, true)                   => KeyEvent::Ignore,
            (b @ 0x20...0x7e, _)        => KeyEvent::Command(ctor(Key::Char(b as u8 as char))),
            (0xff08, _)                 => KeyEvent::Command(ctor(Key::Backspace)),
            (0xff09, _)                 => KeyEvent::Command(ctor(Key::Char('\x09'))),
            (0xff0a, _) | (0xff0d, _)   => KeyEvent::Command(ctor(Key::Enter)),
            (0xff14, _)                 => KeyEvent::Command(ctor(Key::ScrollLock)),
            (0xff1b, _)                 => KeyEvent::Command(ctor(Key::Char('\x1b'))),
            (0xff50, _)                 => KeyEvent::Command(ctor(Key::Home)),
            (0xff51, _)                 => KeyEvent::Command(ctor(Key::LeftArrow)),
            (0xff52, _)                 => KeyEvent::Command(ctor(Key::UpArrow)),
            (0xff53, _)                 => KeyEvent::Command(ctor(Key::RightArrow)),
            (0xff54, _)                 => KeyEvent::Command(ctor(Key::DownArrow)),
            (0xff55, _)                 => KeyEvent::Command(ctor(Key::PageUp)),
            (0xff56, _)                 => KeyEvent::Command(ctor(Key::PageDown)),
            (0xff57, _)                 => KeyEvent::Command(ctor(Key::End)),
            (0xffe1, _)                 => KeyEvent::Command(ctor(Key::ShiftLeft)),
            (0xffe2, _)                 => KeyEvent::Command(ctor(Key::ShiftRight)),
            (0xffe3, _)                 => KeyEvent::Command(ctor(Key::CtrlLeft)),
            (0xffe4, _)                 => KeyEvent::Command(ctor(Key::CtrlRight)),
            (0xffe5, _)                 => KeyEvent::Command(ctor(Key::CapsLock)),
            (0xffe7, _) | (0xffeb, _)   => KeyEvent::Command(ctor(Key::Meta)),
            (0xffe8, _) | (0xff67, _)   => KeyEvent::Command(ctor(Key::Menu)),
            (0xffe9, _)                 => KeyEvent::Command(ctor(Key::AltLeft)),
            (0xffea, _) | (0xfe03, _)   => KeyEvent::Command(ctor(Key::AltGr)),
            (0xffff, _)                 => KeyEvent::Command(ctor(Key::Delete)),
            (x, _)                      => { panic!("Key press: {:x}", x) }
        }
    }
}

// Returns true if this key event is while control and shift are held down
fn shift_ctrl(event: &EventKey) -> bool {
    event.get_state().contains(CONTROL_MASK) && event.get_event_type() == EventType::KeyPress
}
