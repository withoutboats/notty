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
#[derive(Copy, Clone)]
pub struct Modifiers {
    pub shift: bool,
    pub caps: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl Modifiers {
    pub fn new() -> Modifiers {
        Modifiers {
            shift: false,
            caps: false,
            ctrl: false,
            alt: false
        }
    }

    pub fn triplet(&self) -> (bool, bool, bool) {
        (self.shift || self.caps, self.ctrl, self.alt)
    }

}
