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

pub struct ScreenEcho {
    pub settings: EchoSettings,
}

impl ScreenEcho {
    pub fn new(settings: EchoSettings) -> ScreenEcho {
        ScreenEcho { settings: settings }
    }

    pub fn echo(&self, key: Key) -> Option<Box<Command>> {
        match key {
            Char(c) if c == self.settings.lerase as char  => {
                wrap(CommandSeries(vec![
                    Box::new(Move::new(ToEdge(Left))) as Box<Command>,
                    Box::new(Erase::new(CursorRow)) as Box<Command>,
                ]))
            }
            Char(c) if c == self.settings.lnext as char   => unimplemented!(),
            Char(c) if c == self.settings.werase as char  => unimplemented!(),
            Char(c) if c.width().is_some()  => wrap(Put::new_char(c)),
            UpArrow     => wrap(Move::new(To(Up, 1, false))),
            DownArrow   => wrap(Move::new(To(Down, 1, false))),
            LeftArrow   => wrap(Move::new(To(Left, 1, true))),
            RightArrow  => wrap(Move::new(To(Right, 1, true))),
            Enter       => wrap(Move::new(NextLine(1))),
            Backspace   => {
                wrap(CommandSeries(vec![
                    Box::new(Move::new(To(Left, 1, false))) as Box<Command>,
                    Box::new(RemoveChars::new(1)) as Box<Command>,
                ]))
            }
            PageUp      => wrap(Move::new(PreviousLine(25))),
            PageDown    => wrap(Move::new(NextLine(25))),
            Home        => wrap(Move::new(ToBeginning)),
            End         => wrap(Move::new(ToEnd)),
            Delete      => wrap(RemoveChars::new(1)),
            _           => None
        }
    }
}

fn wrap<T: Command>(cmd: T) -> Option<Box<Command>> {
    Some(Box::new(cmd) as Box<Command>)
}
