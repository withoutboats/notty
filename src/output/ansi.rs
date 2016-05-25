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
use std::cell::RefCell;
use std::mem;

use Command;
use command::*;
use datatypes::Code;
use datatypes::args::*;

#[derive(Debug)]
pub struct AnsiData {
    pub private_mode: char,
    pub preterminal: char,
    pub args: Vec<u32>,
    pub arg_buf: String,
}

impl Default for AnsiData {
    fn default() -> AnsiData {
        AnsiData {
            private_mode: '\0',
            preterminal: '\0',
            args: vec![],
            arg_buf: String::new(),
        }
    }
}

impl AnsiData {

    pub fn clear(&mut self) {
        self.private_mode = '\0';
        self.preterminal = '\0';
        self.args.clear();
    }

    pub fn csi(&self, terminal: char) -> Option<Command> {
        macro_rules! command_series {
            ($cmds:expr) => (wrap(CommandSeries(self.args.iter().filter_map($cmds).collect())))
        }
        match (terminal, self.private_mode, self.preterminal) {
            ('@', '\0', '\0')        => wrap(InsertBlank::new(self.arg(0,1))),
            ('A', '\0', '\0')        => wrap(Move::new(To(Up, self.arg(0,1), false))),
            ('B', '\0', '\0')        => wrap(Move::new(To(Down, self.arg(0,1), false))),
            ('C', '\0', '\0')        => wrap(Move::new(To(Right, self.arg(0,1), false))),
            ('D', '\0', '\0')        => wrap(Move::new(To(Left, self.arg(0,1), false))),
            ('E', '\0', '\0')        => wrap(Move::new(NextLine(self.arg(0,1)))),
            ('F', '\0', '\0')        => wrap(Move::new(PreviousLine(self.arg(0,1)))),
            ('G', '\0', '\0')        => wrap(Move::new(Column(self.arg(0,1)-1))),
            ('H', '\0', '\0')        => wrap(Move::new(Position(Coords {
                x: self.arg(1,1)-1,
                y: self.arg(0,1)-1,
            }))),
            ('I', '\0', '\0')        => wrap(Move::new(Tab(Right, self.arg(0,1), false))),
            ('J', '\0', '\0')        => match self.arg(0, 0) {
                0   => wrap(Erase::new(CursorTo(ToEnd))),
                1   => wrap(Erase::new(CursorTo(ToBeginning))),
                2   => wrap(Erase::new(WholeScreen)),
                3   => wrap(NoFeature(self.csi_code(terminal))),
                _   => None
            },
            ('J', '?', '\0')     => wrap(NoFeature(self.csi_code(terminal))),
            ('K', '\0', '\0')        => match self.arg(0, 0) {
                0   => wrap(Erase::new(CursorTo(ToEdge(Right)))),
                1   => wrap(Erase::new(CursorTo(ToEdge(Left)))),
                2   => wrap(Erase::new(CursorRow)),
                _   => None
            },
            ('K', '?', '\0')     => wrap(NoFeature(self.csi_code(terminal))),
            ('L', '\0', '\0')        => wrap(InsertRows::new(self.arg(0,1), true)),
            ('M', '\0', '\0')        => wrap(RemoveRows::new(self.arg(0,1), true)),
            ('P', '\0', '\0')        => wrap(RemoveChars::new(self.arg(0,1))),
            ('S', '\0', '\0')        => wrap(ScrollScreen::new(Down, self.arg(0,1))),
            ('T', '\0', '\0')        => wrap(ScrollScreen::new(Up, self.arg(0,1))),
            ('T', '>', '\0')     => wrap(NoFeature(self.csi_code(terminal))),
            ('X', '\0', '\0')        => wrap(Erase::new(CursorTo(To(Right, self.arg(0,1), false)))),
            ('Z', '\0', '\0')        => wrap(Move::new(Tab(Left, self.arg(0,1), false))),
            ('`', '\0', '\0')        => wrap(Move::new(Column(self.arg(0,1)-1))),
            ('a', '\0', '\0')        => wrap(Move::new(To(Right, self.arg(0,1), false))),
            ('b', '\0', '\0')        => wrap(NoFeature(self.csi_code(terminal))),
            ('c', '\0', '\0')        => wrap(NoFeature(self.csi_code(terminal))),
            ('c', '>', '\0')     => wrap(NoFeature(self.csi_code(terminal))),
            ('d', '\0', '\0')        => wrap(Move::new(Row(self.arg(0,1)-1))),
            ('e', '\0', '\0')        => wrap(Move::new(To(Down, self.arg(0,1), false))),
            ('f', '\0', '\0')        => wrap(Move::new(Position(Coords {
                x: self.arg(1,1)-1,
                y: self.arg(0,1)-1
            }))),
            ('g', '\0', '\0')        => wrap(NoFeature(self.csi_code(terminal))),
            ('h', '\0', '\0')        => command_series!(|x| match *x {
                2   => wrap(NoFeature(self.csi_code(terminal))),
                4   => wrap(NoFeature(self.csi_code(terminal))),
                12  => wrap(NoFeature(self.csi_code(terminal))),
                _   => None,
            }),
            ('h', '?', '\0')     => command_series!(|x| match *x {
                1       => wrap(SetInputMode(Ansi(true))),
                6       => wrap(NoFeature(self.csi_code(terminal))),
                7       => wrap(NoFeature(self.csi_code(terminal))),
                12      => wrap(SetCursorStyle(Blink(true))),
                25      => wrap(SetCursorStyle(Opacity(0))),
                30      => wrap(NoFeature(self.csi_code(terminal))),
                41      => wrap(NoFeature(self.csi_code(terminal))),
                47      => wrap(NoFeature(self.csi_code(terminal))),
                66      => wrap(NoFeature(self.csi_code(terminal))),
                69      => wrap(NoFeature(self.csi_code(terminal))),
                1000    => wrap(NoFeature(self.csi_code(terminal))),
                1001    => wrap(NoFeature(self.csi_code(terminal))),
                1002    => wrap(NoFeature(self.csi_code(terminal))),
                1003    => wrap(NoFeature(self.csi_code(terminal))),
                1004    => wrap(NoFeature(self.csi_code(terminal))),
                1005    => wrap(NoFeature(self.csi_code(terminal))),
                1006    => wrap(NoFeature(self.csi_code(terminal))),
                1007    => wrap(NoFeature(self.csi_code(terminal))),
                1034    => wrap(NoFeature(self.csi_code(terminal))),
                1035    => wrap(NoFeature(self.csi_code(terminal))),
                1036    => wrap(NoFeature(self.csi_code(terminal))),
                1037    => wrap(NoFeature(self.csi_code(terminal))),
                1039    => wrap(NoFeature(self.csi_code(terminal))),
                1040    => wrap(NoFeature(self.csi_code(terminal))),
                1041    => wrap(NoFeature(self.csi_code(terminal))),
                1042    => wrap(NoFeature(self.csi_code(terminal))),
                1043    => wrap(NoFeature(self.csi_code(terminal))),
                1047    => wrap(NoFeature(self.csi_code(terminal))),
                1048    => wrap(NoFeature(self.csi_code(terminal))),
                1049    => wrap(PushPanel(None, Some(false))),
                1050    => wrap(NoFeature(self.csi_code(terminal))),
                2004    => wrap(SetInputMode(BracketedPasteMode(true))),
                _       => None
            }),
            ('i', '\0', '\0')        => wrap(NoFeature(self.csi_code(terminal))),
            ('i', '?', '\0')     => wrap(NoFeature(self.csi_code(terminal))),
            ('l', '\0', '\0')        => command_series!(|x| match *x {
                2   => wrap(NoFeature(self.csi_code(terminal))),
                4   => wrap(NoFeature(self.csi_code(terminal))),
                12  => wrap(NoFeature(self.csi_code(terminal))),
                _   => None,
            }),
            ('l', '?', '\0')      => command_series!(|x| match *x {
                1       => wrap(SetInputMode(Ansi(false))),
                6       => wrap(NoFeature(self.csi_code(terminal))),
                7       => wrap(NoFeature(self.csi_code(terminal))),
                12      => wrap(SetCursorStyle(Blink(false))),
                25      => wrap(SetCursorStyle(Opacity(0xff))),
                30      => wrap(NoFeature(self.csi_code(terminal))),
                41      => wrap(NoFeature(self.csi_code(terminal))),
                47      => wrap(NoFeature(self.csi_code(terminal))),
                66      => wrap(NoFeature(self.csi_code(terminal))),
                69      => wrap(NoFeature(self.csi_code(terminal))),
                1000    => wrap(NoFeature(self.csi_code(terminal))),
                1001    => wrap(NoFeature(self.csi_code(terminal))),
                1002    => wrap(NoFeature(self.csi_code(terminal))),
                1003    => wrap(NoFeature(self.csi_code(terminal))),
                1004    => wrap(NoFeature(self.csi_code(terminal))),
                1005    => wrap(NoFeature(self.csi_code(terminal))),
                1006    => wrap(NoFeature(self.csi_code(terminal))),
                1007    => wrap(NoFeature(self.csi_code(terminal))),
                1034    => wrap(NoFeature(self.csi_code(terminal))),
                1035    => wrap(NoFeature(self.csi_code(terminal))),
                1036    => wrap(NoFeature(self.csi_code(terminal))),
                1037    => wrap(NoFeature(self.csi_code(terminal))),
                1039    => wrap(NoFeature(self.csi_code(terminal))),
                1040    => wrap(NoFeature(self.csi_code(terminal))),
                1041    => wrap(NoFeature(self.csi_code(terminal))),
                1042    => wrap(NoFeature(self.csi_code(terminal))),
                1043    => wrap(NoFeature(self.csi_code(terminal))),
                1047    => wrap(NoFeature(self.csi_code(terminal))),
                1048    => wrap(NoFeature(self.csi_code(terminal))),
                1049    => wrap(PopPanel(None)),
                1050    => wrap(NoFeature(self.csi_code(terminal))),
                2004    => wrap(SetInputMode(BracketedPasteMode(false))),
                _       => None
            }),
            ('m', '\0', '\0')        => match self.arg(0, 0) {
                0               => wrap(DefaultTextStyle),
                38              => match self.arg(1, 0) {
                    2   => match (self.arg(3, 257), self.arg(4, 257), self.arg(5, 257)) {
                        (r, g, b) if r < 256 && g < 256 && b < 256
                            => wrap(SetTextStyle(FgColor(Color::True(r as u8, g as u8, b as u8)))),
                        _   => None
                    },
                    5   => wrap(SetTextStyle(FgColor(Color::Palette(self.arg(2, 0) as u8)))),
                    _   => None
                },
                48              => match self.arg(1, 0) {
                    2   => match (self.arg(3, 257), self.arg(4, 257), self.arg(5, 257)) {
                        (r, g, b) if r < 256 && g < 256 && b < 256
                            => wrap(SetTextStyle(BgColor(Color::True(r as u8, g as u8, b as u8)))),
                        _   => None
                    },
                    5   => wrap(SetTextStyle(BgColor(Color::Palette(self.arg(2, 0) as u8)))),
                    _   => None
                },
                _               => {
                    command_series!(|x| match *x {
                    0               => wrap(DefaultTextStyle),
                    1               => wrap(SetTextStyle(Bold(true))),
                    3               => wrap(SetTextStyle(Italic(true))),
                    4               => wrap(SetTextStyle(Underline(1))),
                    5 | 6           => wrap(SetTextStyle(Blink(true))),
                    7               => wrap(SetTextStyle(InvertColors(true))),
                    8               => wrap(SetTextStyle(Opacity(0))),
                    9               => wrap(SetTextStyle(Strikethrough(true))),
                    21              => wrap(SetTextStyle(Underline(2))),
                    22              => wrap(SetTextStyle(Bold(false))),
                    23              => wrap(SetTextStyle(Italic(false))),
                    24              => wrap(SetTextStyle(Underline(0))),
                    25              => wrap(SetTextStyle(Blink(false))),
                    27              => wrap(SetTextStyle(InvertColors(false))),
                    28              => wrap(SetTextStyle(Opacity(0xff))),
                    29              => wrap(SetTextStyle(Strikethrough(false))),
                    n @ 30...37     => wrap(SetTextStyle(FgColor(Color::Palette((n - 30) as u8)))),
                    39              => wrap(SetTextStyle(FgColor(Color::Default))),
                    n @ 40...47     => wrap(SetTextStyle(BgColor(Color::Palette((n - 40) as u8)))),
                    49              => wrap(SetTextStyle(BgColor(Color::Default))),
                    n @ 90...97     => wrap(SetTextStyle(FgColor(Color::Palette((n - 82) as u8)))),
                    n @ 100...107   => wrap(SetTextStyle(BgColor(Color::Palette((n - 92) as u8)))),
                    _               => None
                })
                }
            },
            ('m', '>', '\0')     => wrap(NoFeature(self.csi_code(terminal))),
            ('n', '\0', '\0')        => match self.arg(0,5) {
                5   => wrap(StaticResponse("\x1b[0n")),
                6   => wrap(ReportPosition(Code::ANSI)),
                _   => None
            },
            ('n', '>', '\0')     => wrap(NoFeature(self.csi_code(terminal))),
            ('n', '?', '\0')     => wrap(NoFeature(self.csi_code(terminal))),
            ('p', '\0', '!')     => wrap(NoFeature(self.csi_code(terminal))),
            ('p', '\0', '$')     => wrap(NoFeature(self.csi_code(terminal))),
            ('p', '\0', '"')     => wrap(NoFeature(self.csi_code(terminal))),
            ('p', '>', '\0')     => wrap(NoFeature(self.csi_code(terminal))),
            ('p', '?', '$')  => wrap(NoFeature(self.csi_code(terminal))),
            ('q', '\0', '\0')        => wrap(NoFeature(self.csi_code(terminal))),
            ('q', '\0', ' ')     => match self.arg(0,1) {
                0 | 1   => wrap(NoFeature(self.csi_code(terminal))),
                2       => wrap(NoFeature(self.csi_code(terminal))),
                3       => wrap(NoFeature(self.csi_code(terminal))),
                4       => wrap(NoFeature(self.csi_code(terminal))),
                5       => wrap(NoFeature(self.csi_code(terminal))),
                6       => wrap(NoFeature(self.csi_code(terminal))),
                _       => None,
            },
            ('q', '\0', '"')     => wrap(NoFeature(self.csi_code(terminal))),
            ('r', '\0', '\0')        => wrap(NoFeature(self.csi_code(terminal))),
            ('r', '\0', '$')     => {
                let area = match (self.arg(0,0), self.arg(1,0), self.arg(2,0), self.arg(3,0)) {
                    (0, _, _, _) | (_, 0, _, _) | (_, _, 0, _) | (_, _, _, 0)   => WholeScreen,
                    (t, l, b, r)    => Bound(Region::new(l-1, t-1, r-1, b-1))
                };
                match self.arg(4,0) {
                    0               => wrap(DefaultStyleInArea(area)),
                    1               => wrap(SetStyleInArea(area,  Bold(true))),
                    3               => wrap(SetStyleInArea(area,  Italic(true))),
                    4               => wrap(SetStyleInArea(area,  Underline(1))),
                    5 | 6           => wrap(SetStyleInArea(area,  Blink(true))),
                    7               => wrap(SetStyleInArea(area,  InvertColors(true))),
                    8               => wrap(SetStyleInArea(area,  Opacity(0))),
                    9               => wrap(SetStyleInArea(area,  Strikethrough(true))),
                    21              => wrap(SetStyleInArea(area,  Underline(2))),
                    22              => wrap(SetStyleInArea(area,  Bold(false))),
                    23              => wrap(SetStyleInArea(area,  Italic(false))),
                    24              => wrap(SetStyleInArea(area,  Underline(0))),
                    25              => wrap(SetStyleInArea(area,  Blink(false))),
                    27              => wrap(SetStyleInArea(area,  InvertColors(false))),
                    28              => wrap(SetStyleInArea(area,  Opacity(0xff))),
                    29              => wrap(SetStyleInArea(area,  Strikethrough(false))),
                    _               => None,
                }
            }
            ('r', '?', '\0')     => wrap(NoFeature(self.csi_code(terminal))),
            ('s', '\0', '\0')        => wrap(NoFeature(self.csi_code(terminal))), //left and right margins
            ('s', '?', '\0')     => wrap(NoFeature(self.csi_code(terminal))),
            ('t', '\0', '\0')        => wrap(NoFeature(self.csi_code(terminal))), //window manipulation
            ('t', '\0', ' ')     => wrap(NoFeature(self.csi_code(terminal))),
            ('t', '\0', '$')     => wrap(NoFeature(self.csi_code(terminal))), // DECRARA
            ('t', '>', '\0')     => wrap(NoFeature(self.csi_code(terminal))),
            ('u', '\0', '\0')        => wrap(NoFeature(self.csi_code(terminal))), // Restore cursor?
            ('u', '\0', ' ')     => wrap(NoFeature(self.csi_code(terminal))),
            ('v', '\0', '$')     => wrap(NoFeature(self.csi_code(terminal))), // Copy an area
            ('w', '\0', '\'')    => wrap(NoFeature(self.csi_code(terminal))),
            ('x', '\0', '\0')        => wrap(NoFeature(self.csi_code(terminal))),
            ('x', '\0', '*')     => wrap(NoFeature(self.csi_code(terminal))),
            ('x', '\0', '$')     => wrap(NoFeature(self.csi_code(terminal))),
            ('y', '\0', '*')     => wrap(NoFeature(self.csi_code(terminal))),
            ('z', '\0', '$')     => wrap(NoFeature(self.csi_code(terminal))), // erase rectangular area
            ('z', '\0', '\'')    => wrap(NoFeature(self.csi_code(terminal))),
            ('{', '\0', '\'')    => wrap(NoFeature(self.csi_code(terminal))),
            ('{', '\0', '$')     => wrap(NoFeature(self.csi_code(terminal))),
            ('|', '\0', '\'')    => wrap(NoFeature(self.csi_code(terminal))),
            ('}', '\0', '\'')    => wrap(NoFeature(self.csi_code(terminal))), 
            ('~', '\0', '\'')    => wrap(NoFeature(self.csi_code(terminal))), 
            _                   => None
        }
    }

    #[allow(unused, dead_code)]
    pub fn dcs(&self, strarg: &str) -> Option<Command> {
        match (self.private_mode, self.preterminal) {
            ('|', '\0')   => unimplemented!(),
            ('$', 'q')    => unimplemented!(),
            ('+', 'p')    => unimplemented!(),
            ('+', 'q')    => unimplemented!(),
            _               => unreachable!(),
        }
    }

    pub fn osc(&mut self) -> Option<Command> {
        match self.arg(0, 0) {
            0...2   =>  {
                let title = mem::replace(&mut self.arg_buf, String::new());
                wrap(SetTitle(RefCell::new(Some(title))))
            }
            3   => unimplemented!(),
            4   => unimplemented!(),
            5   => unimplemented!(),
            6   => unimplemented!(),
            46  => unimplemented!(),
            50  => unimplemented!(),
            51  => unimplemented!(),
            52  => unimplemented!(),
            104 => unimplemented!(),
            105 => unimplemented!(),
            106 => unimplemented!(),
            _   => None
        }
    }

    fn arg(&self, idx: usize, default: u32) -> u32 {
        self.args.get(idx).map_or(default, |&x|x)
    }

    fn csi_code(&self, terminal: char) -> String {
        let args = self.args.iter().map(ToString::to_string).collect::<Vec<_>>().join(";");
        format!("^[[{}{}{}{}", self.private_mode, args, self.preterminal, terminal)
    }

}

fn wrap<T: CommandTrait>(cmd: T) -> Option<Command> {
    Some(Command { inner: Box::new(cmd) as Box<CommandTrait> })
}
