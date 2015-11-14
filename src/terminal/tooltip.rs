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
use std::fmt;

use datatypes::Key;

use self::Tooltip::*;

pub enum Tooltip {
    Basic(String),
    Menu {
        options: Vec<String>,
        position: Option<usize>,
    }
}

impl Tooltip {
    pub fn interact(&mut self, key: &Key) -> Result<usize, bool> {
        match self {
            &mut Menu { ref mut position, .. }   => match (key, position.take()) {
                (&Key::Down, None)      => {
                    *position = Some(0);
                    Err(false)
                }
                (&Key::Down, Some(n))   => {
                    *position = Some(n + 1);
                    Err(false)
                }
                (&Key::Up, Some(0))     => Err(false),
                (&Key::Up, Some(n))     => {
                    *position = Some(n - 1);
                    Err(false)
                }
                (&Key::Enter, Some(n))  => Ok(n),
                _   => Err(true)
            },
            _   => Err(true)
        }
    }
}

impl fmt::Display for Tooltip {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Basic(ref s)                => f.write_str(s),
            Menu { ref options, .. }    => f.write_str(&options.join("\n")),
        }
    }
}
