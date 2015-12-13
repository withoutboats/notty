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
use unicode_width::*;

use command::*;
use datatypes::{EchoSettings, Key};
use datatypes::Key::*;
use datatypes::Area::*;
use datatypes::Movement::*;
use datatypes::Direction::*;

pub struct LineEcho {
    pub settings: EchoSettings,
    position: u32,
    len: u32,
}

impl LineEcho {
    pub fn new(settings: EchoSettings) -> LineEcho {
        LineEcho {
            settings: settings,
            position: 0,
            len: 0,
        }
    }

    pub fn echo(&mut self, key: Key) -> Option<Box<Command>> {
        match key {
            Char(c) if c == self.settings.lerase as char  => {
                self.len = 0;
                wrap(CommandSeries(vec![
                    Box::new(Move::new(To(Left, self.position, true))) as Box<Command>,
                    Box::new(Erase::new(CursorTo(To(Right, self.len, true)))) as Box<Command>,
                ]))
            }
            Char(c) if c == self.settings.lnext as char   => unimplemented!(),
            Char(c) if c == self.settings.werase as char  => unimplemented!(),
            Char(c) if c.width().is_some() => {
                self.len += 1;
                wrap(Put::new_char(c))
            }
            LeftArrow if self.position != 0 => {
                self.position -= 1;
                wrap(Move::new(To(Left, 1, false)))
            }
            RightArrow if self.position != self.len => {
                self.position += 1;
                wrap(Move::new(To(Right, 1, false)))
            }
            Enter       => wrap(Move::new(NextLine(1))),
            Backspace if self.position != 0 => {
                self.len -= 1;
                wrap(CommandSeries(vec![
                    Box::new(Move::new(To(Left, 1, false))) as Box<Command>,
                    Box::new(RemoveChars::new(1)) as Box<Command>,
                ]))
            }
            Delete if self.position != self.len => {
                self.len -= 1;
                wrap(RemoveChars::new(1))
            }
            Home        => {
                self.position = 0;
                wrap(Move::new(To(Left, self.position, true)))
            }
            _           => None
        }
    }
}

fn wrap<T: Command>(cmd: T) -> Option<Box<Command>> {
    Some(Box::new(cmd) as Box<Command>)
}
