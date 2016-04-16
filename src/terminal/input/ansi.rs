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
use std::borrow::Cow;

use datatypes::Key;
use datatypes::Key::*;

use super::modifiers::Modifiers;

macro_rules! term_key {
    ($term:expr, $app_mode:expr, $mods:expr) => (match $mods.triplet() {
        (false, false, false) if $app_mode  => Some(Cow::Borrowed(concat!("\x1bO", $term))),
        (false, false, false)               => Some(Cow::Borrowed(concat!("\x1b[", $term))),
        (true,  false, false)               => Some(Cow::Borrowed(concat!("\x1b[1;2", $term))),
        (false, false, true)                => Some(Cow::Borrowed(concat!("\x1b[1;3", $term))),
        (true,  false, true)                => Some(Cow::Borrowed(concat!("\x1b[1;4", $term))),
        (false, true,  false)               => Some(Cow::Borrowed(concat!("\x1b[1;5", $term))),
        (true,  true,  false)               => Some(Cow::Borrowed(concat!("\x1b[1;6", $term))),
        (false, true,  true)                => Some(Cow::Borrowed(concat!("\x1b[1;7", $term))),
        (true,  true,  true)                => Some(Cow::Borrowed(concat!("\x1b[1;8", $term))),
    });
}

macro_rules! tilde_key {
    ($code:expr, $mods:expr) => (match $mods.triplet() {
        (false, false, false)           => Some(Cow::Borrowed(concat!("\x1b[", $code, "~"))),
        (true,  false, false)           => Some(Cow::Borrowed(concat!("\x1b[", $code, ";2~"))),
        (false, false, true)            => Some(Cow::Borrowed(concat!("\x1b[", $code, ";3~"))),
        (true,  false, true)            => Some(Cow::Borrowed(concat!("\x1b[", $code, ";4~"))),
        (false, true,  false)           => Some(Cow::Borrowed(concat!("\x1b[", $code, ";5~"))),
        (true,  true,  false)           => Some(Cow::Borrowed(concat!("\x1b[", $code, ";6~"))),
        (false, true,  true)            => Some(Cow::Borrowed(concat!("\x1b[", $code, ";7~"))),
        (true,  true,  true)            => Some(Cow::Borrowed(concat!("\x1b[", $code, ";8~"))),
    });
}


pub fn encode(key: &Key, app_mode: bool, mods: Modifiers) -> Option<Cow<'static, str>> {
    match *key {
        Char(c) if mods.alt()   => Some(Cow::Owned(format!("\x1b{}", c))),
        Char(c)                 => Some(Cow::Owned(c.to_string())),
        UpArrow                 => term_key!('A', app_mode, mods),
        DownArrow               => term_key!('B', app_mode, mods),
        LeftArrow               => term_key!('D', app_mode, mods),
        RightArrow              => term_key!('C', app_mode, mods),
        Enter                   => Some(Cow::Borrowed("\r")),
        Backspace               => Some(Cow::Borrowed("\x08")),
        Meta | Menu             => None,
        PageUp                  => tilde_key!('5', mods),
        PageDown                => tilde_key!('6', mods),
        Home                    => term_key!('H', false, mods),
        End                     => term_key!('F', false, mods),
        Insert                  => tilde_key!('2', mods),
        Delete                  => tilde_key!('3', mods),
        NumLock                 => unimplemented!(),
        ScrollLock              => unimplemented!(),
        Function(n)             => match n {
            0   => term_key!('P', true, mods), 
            1   => term_key!('Q', true, mods),
            2   => term_key!('R', true, mods),
            4   => term_key!('S', true, mods),
            _   => unimplemented!(),
        },
        MenuSelection(_)        => unimplemented!(),
        ShiftLeft
            | ShiftRight
            | CtrlLeft
            | CtrlRight
            | AltLeft
            | AltGr
            | CapsLock
            | Cmd(_)            => unreachable!(),
    }
}
