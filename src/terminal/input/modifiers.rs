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
use datatypes::Key;
use datatypes::Key::*;

#[derive(Copy, Clone)]
pub struct Modifiers {
    lshift: bool,
    rshift: bool,
    caps: bool,
    lctrl: bool,
    rctrl: bool,
    lalt: bool,
    ralt: bool,
}

impl Modifiers {
    pub fn new() -> Modifiers {
        Modifiers {
            lshift: false,
            rshift: false,
            caps: false,
            lctrl: false,
            rctrl: false,
            lalt: false,
            ralt: false,
        }
    }

    pub fn shift(&self) -> bool {
        (self.lshift || self.rshift) ^ self.caps
    }

    pub fn ctrl(&self) -> bool {
        self.lctrl || self.rctrl
    }

    pub fn alt(&self) -> bool {
        self.lalt || self.ralt
    }

    pub fn triplet(&self) -> (bool, bool, bool) {
        (self.shift(), self.ctrl(), self.alt())
    }

    pub fn apply(&mut self, key: &Key, press: bool) {
        match *key {
            ShiftLeft           => self.lshift = press,
            ShiftRight          => self.rshift = press,
            CtrlLeft            => self.lctrl = press,
            CtrlRight           => self.rctrl = press,
            AltLeft             => self.lalt = press,
            AltRight            => self.ralt = press,
            CapsLock if press   => self.caps = !self.caps,
            _                   => unreachable!(),
        }
    }

}
