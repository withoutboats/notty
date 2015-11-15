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

macro_rules! key {
    ($k:expr, $press:expr, $mods:expr) =>(match ($mods.triplet(), $press) {
        ((false, false, false), false)  => Cow::Borrowed(concat!("\x1b{0;",$k,"}")),
        ((false, false, false), true)   => Cow::Borrowed(concat!("\x1b{1;",$k,"}")),
        ((false, false, true),  false)  => Cow::Borrowed(concat!("\x1b{2;",$k,"}")),
        ((false, false, true),  true)   => Cow::Borrowed(concat!("\x1b{3;",$k,"}")),
        ((false, true,  false), false)  => Cow::Borrowed(concat!("\x1b{4;",$k,"}")),
        ((false, true,  false), true)   => Cow::Borrowed(concat!("\x1b{5;",$k,"}")),
        ((false, true,  true),  false)  => Cow::Borrowed(concat!("\x1b{6;",$k,"}")),
        ((false, true,  true),  true)   => Cow::Borrowed(concat!("\x1b{7;",$k,"}")),
        ((true,  false, false), false)  => Cow::Borrowed(concat!("\x1b{8;",$k,"}")),
        ((true,  false, false), true)   => Cow::Borrowed(concat!("\x1b{9;",$k,"}")),
        ((true,  false, true),  false)  => Cow::Borrowed(concat!("\x1b{a;",$k,"}")),
        ((true,  false, true),  true)   => Cow::Borrowed(concat!("\x1b{b;",$k,"}")),
        ((true,  true,  false), false)  => Cow::Borrowed(concat!("\x1b{c;",$k,"}")),
        ((true,  true,  false), true)   => Cow::Borrowed(concat!("\x1b{d;",$k,"}")),
        ((true,  true,  true),  false)  => Cow::Borrowed(concat!("\x1b{e;",$k,"}")),
        ((true,  true,  true),  true)   => Cow::Borrowed(concat!("\x1b{f;",$k,"}")),
    });
}

pub fn encode(key: Key, press: bool, _: (), mods: Modifiers) -> Cow<'static, str> {
    match key {
        Char(c)             => char_key(c, press, mods),
        Enter               => char_key('\n', press, mods),
        Delete              => char_key('\x7f', press, mods),
        UpArrow             => key!('1', press, mods),
        DownArrow           => key!('2', press, mods),
        LeftArrow           => key!('3', press, mods),
        RightArrow          => key!('4', press, mods),
        PageUp              => key!('5', press, mods),
        PageDown            => key!('6', press, mods),
        Home                => key!('7', press, mods),
        End                 => key!('8', press, mods),
        Insert              => key!('9', press, mods),
        ShiftLeft           => key!("a", press, mods),
        ShiftRight          => key!("a", press, mods),
        CtrlLeft            => key!("b", press, mods),
        CtrlRight           => key!("b", press, mods),
        AltLeft             => key!("c", press, mods),
        AltGr               => key!("d", press, mods),
        Meta                => key!("e", press, mods),
        Menu                => key!("f", press, mods),
        NumLock             => unimplemented!(),
        ScrollLock          => unimplemented!(),
        CapsLock            => unimplemented!(), 
        Function(_)         => unimplemented!(),
        Cmd(s)              => s,
        MenuSelection(_)    => unimplemented!(),
    }
}

fn char_key(c: char, press: bool, mods: Modifiers) -> Cow<'static, str> {
    match (mods.triplet(), press) {
        ((false, false, false), false)  => Cow::Owned(c.to_string()),
        ((false, false, false), true)   => Cow::Owned(format!("\x1b{{1{{{}}}", c)),
        ((false, false, true),  false)  => Cow::Owned(format!("\x1b{{2{{{}}}", c)),
        ((false, false, true),  true)   => Cow::Owned(format!("\x1b{{3{{{}}}", c)),
        ((false, true,  false), false)  => Cow::Owned(format!("\x1b{{4{{{}}}", c)),
        ((false, true,  false), true)   => Cow::Owned(format!("\x1b{{5{{{}}}", c)),
        ((false, true,  true),  false)  => Cow::Owned(format!("\x1b{{6{{{}}}", c)),
        ((false, true,  true),  true)   => Cow::Owned(format!("\x1b{{7{{{}}}", c)),
        ((true,  false, false), false)  => match c {
            c @ '\x40'...'\x7f' => Cow::Owned((((c as u8) & 0x1f) as char).to_string()),
            c                   => Cow::Owned(format!("\x1b{{8{{{}}}", c)),
        },
        ((true,  false, false), true)   => Cow::Owned(format!("\x1b{{9{{{}}}", c)),
        ((true,  false, true),  false)  => Cow::Owned(format!("\x1b{{a{{{}}}", c)),
        ((true,  false, true),  true)   => Cow::Owned(format!("\x1b{{b{{{}}}", c)),
        ((true,  true,  false), false)  => Cow::Owned(format!("\x1b{{c{{{}}}", c)),
        ((true,  true,  false), true)   => Cow::Owned(format!("\x1b{{d{{{}}}", c)),
        ((true,  true,  true),  false)  => Cow::Owned(format!("\x1b{{e{{{}}}", c)),
        ((true,  true,  true),  true)   => Cow::Owned(format!("\x1b{{f{{{}}}", c)),
    }
}
