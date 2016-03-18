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

use Command;
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

    pub fn echo(&mut self, key: Key) -> Option<Command> {
        match key {
            Char(c) if c == self.settings.lerase as char  => {
                self.len = 0;
                wrap(CommandSeries(vec![
                    Command {
                        inner: Box::new(Move::new(To(Left, self.position, true)))
                               as Box<CommandTrait>
                    },
                    Command {
                        inner: Box::new(Erase::new(CursorTo(To(Right, self.len, true))))
                               as Box<CommandTrait>
                    },
                ]))
            }
            Char(c) if c == self.settings.lnext as char   => unimplemented!(),
            Char(c) if c == self.settings.werase as char  => unimplemented!(),
            Char(c) if c.width().is_some() => {
                self.position += 1;
                self.len += 1;
                wrap(Put::new_char(c))
            }
            LeftArrow if self.position != 0 => {
                self.position -= 1;
                wrap(Move::new(To(Left, 1, true)))
            }
            RightArrow if self.position != self.len => {
                self.position += 1;
                wrap(Move::new(To(Right, 1, true)))
            }
            Enter       => {
                self.position = 0;
                self.len = 0;
                wrap(Move::new(NextLine(1)))
            }
            Backspace if self.position != 0 => {
                self.position -= 1;
                self.len -= 1;
                wrap(CommandSeries(vec![
                    Command {
                        inner: Box::new(Move::new(To(Left, 1, false))) as Box<CommandTrait>
                    },
                    Command { inner: Box::new(RemoveChars::new(1)) as Box<CommandTrait> },
                ]))
            }
            Delete if self.position != self.len => {
                self.len -= 1;
                wrap(RemoveChars::new(1))
            }
            Home if self.position != 0 => {
                self.position = 0;
                wrap(Move::new(To(Left, self.position, true)))
            }
            End if self.position != self.len => {
                let n = self.len - self.position;
                self.position = self.len;
                wrap(Move::new(To(Right, n, true)))
            }
            _           => None
        }
    }
}

fn wrap<T: CommandTrait>(cmd: T) -> Option<Command> {
    Some(Command { inner: Box::new(cmd) as Box<CommandTrait> })
}

#[cfg(test)]
mod tests {

    use command::*;
    use datatypes::EchoSettings;
    use datatypes::Key::*;
    use datatypes::Area::*;
    use datatypes::Movement::*;
    use datatypes::Direction::*;

    use super::*;

    static SETTINGS: EchoSettings = EchoSettings { lerase: 0, lnext: 1, werase: 2 };

    #[test]
    fn chars() {
        let mut echo = LineEcho::new(SETTINGS);
        assert_eq!(echo.echo(Char('A')).unwrap().inner.repr(), Put::new_char('A').repr());
        assert_eq!(echo.len, 1);
        assert_eq!(echo.position, 1);
        assert!(echo.echo(Char('\x1b')).is_none());
        assert_eq!(echo.len, 1);
        assert_eq!(echo.position, 1);
    }

    #[test]
    fn left_arrow() {
        let mut echo = LineEcho { settings: SETTINGS, position: 0, len: 0 };
        assert!(echo.echo(LeftArrow).is_none());
        assert_eq!(echo.len, 0);
        assert_eq!(echo.position, 0);
        let mut echo = LineEcho { settings: SETTINGS, position: 1, len: 1 };
        assert_eq!(echo.echo(LeftArrow).unwrap().inner.repr(), Move::new(To(Left, 1, true)).repr());
        assert_eq!(echo.len, 1);
        assert_eq!(echo.position, 0);
    }

    #[test]
    fn right_arrow() {
        let mut echo = LineEcho { settings: SETTINGS, position: 0, len: 0 };
        assert!(echo.echo(RightArrow).is_none());
        assert_eq!(echo.len, 0);
        assert_eq!(echo.position, 0);
        let mut echo = LineEcho { settings: SETTINGS, position: 0, len: 1 };
        assert_eq!(echo.echo(RightArrow).unwrap().inner.repr(), Move::new(To(Right, 1, true)).repr());
        assert_eq!(echo.len, 1);
        assert_eq!(echo.position, 1);
    }

    #[test]
    fn enter() {
        let mut echo = LineEcho { settings: SETTINGS, position: 3, len: 3 };
        assert_eq!(echo.echo(Enter).unwrap().inner.repr(), Move::new(NextLine(1)).repr());
        assert_eq!(echo.len, 0);
        assert_eq!(echo.position, 0);
    }

    #[test]
    fn backspace() {
        let mut echo = LineEcho { settings: SETTINGS, position: 0, len: 1 };
        assert!(echo.echo(Backspace).is_none());
        assert_eq!(echo.position, 0);
        assert_eq!(echo.len, 1);
        let mut echo = LineEcho { settings: SETTINGS, position: 1, len: 1 };
        assert!(echo.echo(Backspace).is_some());
        assert_eq!(echo.position, 0);
        assert_eq!(echo.len, 0);
    }

    #[test]
    fn delete() {
        let mut echo = LineEcho { settings: SETTINGS, position: 0, len: 1 };
        assert!(echo.echo(Delete).is_some());
        assert_eq!(echo.position, 0);
        assert_eq!(echo.len, 0);
        let mut echo = LineEcho { settings: SETTINGS, position: 1, len: 1 };
        assert!(echo.echo(Delete).is_none());
        assert_eq!(echo.position, 1);
        assert_eq!(echo.len, 1);
    }

    #[test]
    fn home() {
        let mut echo = LineEcho { settings: SETTINGS, position: 4, len: 5 };
        assert!(echo.echo(Home).is_some());
        assert_eq!(echo.position, 0);
        assert_eq!(echo.len, 5);
    }

    #[test]
    fn end() {
        let mut echo = LineEcho { settings: SETTINGS, position: 4, len: 5 };
        assert!(echo.echo(End).is_some());
        assert_eq!(echo.position, 5);
        assert_eq!(echo.len, 5);
    }

}
