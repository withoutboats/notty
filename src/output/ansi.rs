use std::cell::RefCell;

use command::*;
use datatypes::Code;
use datatypes::args::*;

#[derive(Debug)]
pub struct AnsiCode {
    pub private_mode: u8,
    pub preterminal: u8,
    pub terminal: u8,
    pub args: Vec<u32>,
}

impl Default for AnsiCode {
    fn default() -> AnsiCode {
        AnsiCode {
            private_mode: 0,
            preterminal: 0,
            terminal: 0,
            args: vec![],
        }
    }
}

impl AnsiCode {

    pub fn clear(&mut self) {
        self.private_mode = 0;
        self.preterminal = 0;
        self.terminal = 0;
        self.args.clear();
    }

    pub fn csi(&self) -> Option<Box<Command>> {
        macro_rules! command_series {
            ($cmds:expr) => (wrap(CommandSeries(self.args.iter().filter_map($cmds).collect())))
        }
        match (self.terminal, self.private_mode, self.preterminal) {
            (b'@', 0, 0)        => wrap(InsertBlank::new(self.arg(0,1))),
            (b'A', 0, 0)        => wrap(Move::new(Up(self.arg(0,1)), )),
            (b'B', 0, 0)        => wrap(Move::new(Down(self.arg(0,1)), )),
            (b'C', 0, 0)        => wrap(Move::new(Right(self.arg(0,1)), )),
            (b'D', 0, 0)        => wrap(Move::new(Left(self.arg(0,1)), )),
            (b'E', 0, 0)        => wrap(Move::new(NextLine(self.arg(0,1)), )),
            (b'F', 0, 0)        => wrap(Move::new(PreviousLine(self.arg(0,1)), )),
            (b'G', 0, 0)        => wrap(Move::new(Column(self.arg(0,1)-1), )),
            (b'H', 0, 0)        => wrap(Move::new(Position(Coords::new(self.arg(1,1)-1,
                                                                       self.arg(0,1)-1)),
                                        )),
            (b'I', 0, 0)        => wrap(Move::new(RightTab(self.arg(0,1)), )),
            (b'J', 0, 0)        => match self.arg(0, 0) {
                0   => wrap(Erase::new(CursorTo(ToEnd))),
                1   => wrap(Erase::new(CursorTo(ToBeginning))),
                2   => wrap(Erase::new(WholeScreen)),
                3   => wrap(NoFeature(self.csi_code())),
                _   => None
            },
            (b'J', b'?', 0)     => wrap(NoFeature(self.csi_code())),
            (b'K', 0, 0)        => match self.arg(0, 0) {
                0   => wrap(Erase::new(CursorTo(RightToEdge))),
                1   => wrap(Erase::new(CursorTo(LeftToEdge))),
                2   => wrap(Erase::new(CursorRow)),
                _   => None
            },
            (b'K', b'?', 0)     => wrap(NoFeature(self.csi_code())),
            (b'L', 0, 0)        => wrap(InsertRows::new(self.arg(0,1), true)),
            (b'M', 0, 0)        => wrap(RemoveRows::new(self.arg(0,1), true)),
            (b'P', 0, 0)        => wrap(RemoveChars::new(self.arg(0,1))),
            (b'S', 0, 0)        => wrap(ScrollScreen::new(Down(self.arg(0,1)))),
            (b'T', 0, 0)        => wrap(ScrollScreen::new(Up(self.arg(0,1)))),
            (b'T', b'>', 0)     => wrap(NoFeature(self.csi_code())),
            (b'X', 0, 0)        => wrap(Erase::new(CursorTo(Right(self.arg(0,1))))),
            (b'Z', 0, 0)        => wrap(Move::new(LeftTab(self.arg(0,1)), )),
            (b'`', 0, 0)        => wrap(Move::new(Column(self.arg(0,1)-1), )),
            (b'a', 0, 0)        => wrap(Move::new(Right(self.arg(0,1)), )),
            (b'b', 0, 0)        => wrap(NoFeature(self.csi_code())),
            (b'c', 0, 0)        => unimplemented!(),
            (b'c', b'>', 0)     => wrap(NoFeature(self.csi_code())),
            (b'd', 0, 0)        => wrap(Move::new(Row(self.arg(0,1)-1), )),
            (b'e', 0, 0)        => wrap(Move::new(Down(self.arg(0,1)), )),
            (b'f', 0, 0)        => wrap(Move::new(Position(Coords::new(self.arg(1,1)-1,
                                                                       self.arg(0,1)-1)),
                                        )),
            (b'g', 0, 0)        => wrap(NoFeature(self.csi_code())),
            (b'h', 0, 0)        => command_series!(|x| match *x {
                2   => panic!("not yet implemented: mode {}", 2), // turn off keyboard
                4   => panic!("not yet implemented: mode {}", 4), // insert vs overwrite
                12  => panic!("not yet implemented: mode {}", 12), // local echo
                _   => None,
            }),
            (b'h', b'?', 0)     => command_series!(|x| match *x {
                1       => wrap(SetInputMode(InputMode::Application)),
                6       => panic!("not yet implemented: mode {}", 6), // DECOM
                7       => panic!("not yet implemented: mode {}", 7), // DECAWM
                12      => wrap(SetCursorStyle(Blink(true))),
                25      => wrap(SetCursorStyle(Opacity(0))),
                30      => panic!("not yet implemented: mode {}", 30), // Scroll bar
                41      => panic!("not yet implemented: mode {}", 41), // more() fix XXX
                47      => panic!("not yet implemented: mode {}", 47), // alternate screen buffer???
                66      => panic!("not yet implemented: mode {}", 66), // DECNKM
                69      => panic!("not yet implemented: mode {}", 69), // DECLRMM
                1000    => panic!("not yet implemented: mode {}", 1000), // Mouse support
                1001    => panic!("not yet implemented: mode {}", 1001), // ..
                1002    => panic!("not yet implemented: mode {}", 1002), // ..
                1003    => panic!("not yet implemented: mode {}", 1003), // ..
                1004    => panic!("not yet implemented: mode {}", 1004), // ..
                1005    => panic!("not yet implemented: mode {}", 1005), // ..
                1006    => panic!("not yet implemented: mode {}", 1006), // ..
                1007    => panic!("not yet implemented: mode {}", 1007), // ..
                1034    => panic!("not yet implemented: mode {}", 1034), // modifier key?
                1035    => panic!("not yet implemented: mode {}", 1035), // ..
                1036    => panic!("not yet implemented: mode {}", 1036), // ..
                1037    => panic!("not yet implemented: mode {}", 1037), // ..
                1039    => panic!("not yet implemented: mode {}", 1039), // ..
                1040    => panic!("not yet implemented: mode {}", 1040), // keep selection?
                1041    => panic!("not yet implemented: mode {}", 1041), // clipboard
                1042    => panic!("not yet implemented: mode {}", 1042), // bell
                1043    => panic!("not yet implemented: mode {}", 1043), // bell
                1047    => panic!("not yet implemented: mode {}", 1047), // alternative screen buffer
                1048    => panic!("not yet implemented: mode {}", 1048), // decsc
                1049    => wrap(PushBuffer(false)),
                1050    => panic!("not yet implemented: mode {}", 1050), // something to do with the keyboard
                2004    => panic!("not yet implemented: mode {}", 2004), // bracketed paste
                _       => None
            }),
            (b'i', 0, 0)        => wrap(NoFeature(self.csi_code())),
            (b'i', b'?', 0)     => wrap(NoFeature(self.csi_code())),
            (b'l', 0, 0)        => command_series!(|x| match *x {
                2   => panic!("not yet implemented: mode {}", 2),
                4   => panic!("not yet implemented: mode {}", 4),
                12  => panic!("not yet implemented: mode {}", 12),
                _   => None,
            }),
            (b'l', b'?', 0)      => command_series!(|x| match *x {
                1       => wrap(SetInputMode(InputMode::Ansi)),
                6       => panic!("not yet implemented: mode {}", 6),
                7       => panic!("not yet implemented: mode {}", 7),
                12      => wrap(SetCursorStyle(Blink(false))),
                25      => wrap(SetCursorStyle(Opacity(0xff))),
                30      => panic!("not yet implemented: mode {}", 30),
                41      => panic!("not yet implemented: mode {}", 41),
                47      => panic!("not yet implemented: mode {}", 47),
                66      => panic!("not yet implemented: mode {}", 66),
                69      => panic!("not yet implemented: mode {}", 69),
                1000    => panic!("not yet implemented: mode {}", 1000),
                1001    => panic!("not yet implemented: mode {}", 1001),
                1002    => panic!("not yet implemented: mode {}", 1002),
                1003    => panic!("not yet implemented: mode {}", 1003),
                1004    => panic!("not yet implemented: mode {}", 1004),
                1005    => panic!("not yet implemented: mode {}", 1005),
                1006    => panic!("not yet implemented: mode {}", 1006),
                1007    => panic!("not yet implemented: mode {}", 1007),
                1034    => panic!("not yet implemented: mode {}", 1034),
                1035    => panic!("not yet implemented: mode {}", 1035),
                1036    => panic!("not yet implemented: mode {}", 1036),
                1037    => panic!("not yet implemented: mode {}", 1037),
                1039    => panic!("not yet implemented: mode {}", 1039),
                1040    => panic!("not yet implemented: mode {}", 1040),
                1041    => panic!("not yet implemented: mode {}", 1041),
                1042    => panic!("not yet implemented: mode {}", 1042),
                1043    => panic!("not yet implemented: mode {}", 1043),
                1047    => panic!("not yet implemented: mode {}", 1047),
                1048    => panic!("not yet implemented: mode {}", 1048),
                1049    => wrap(PopBuffer),
                1050    => panic!("not yet implemented: mode {}", 1050),
                2004    => panic!("not yet implemented: mode {}", 2004),
                _       => None
            }),
            (b'm', 0, 0)        => match self.arg(0, 0) {
                0               => wrap(DefaultTextStyle),
                38              => match self.arg(1, 0) {
                    2   => match (self.arg(3, 257), self.arg(4, 257), self.arg(5, 257)) {
                        (r, g, b) if r < 256 && g < 256 && b < 256
                            => wrap(SetTextStyle(FgColor(Color(r as u8, g as u8, b as u8)))),
                        _   => None
                    },
                    5   => wrap(SetTextStyle(FgColorCfg(Some(self.arg(2, 0) as u8)))),
                    _   => None
                },
                48              => match self.arg(1, 0) {
                    2   => match (self.arg(3, 257), self.arg(4, 257), self.arg(5, 257)) {
                        (r, g, b) if r < 256 && g < 256 && b < 256
                            => wrap(SetTextStyle(BgColor(Color(r as u8, g as u8, b as u8)))),
                        _   => None
                    },
                    5   => wrap(SetTextStyle(BgColorCfg(Some(self.arg(2, 0) as u8)))),
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
                    n @ 30...37     => wrap(SetTextStyle(FgColorCfg(Some((n - 30) as u8)))),
                    39              => wrap(SetTextStyle(FgColorCfg(None))),
                    n @ 40...47     => wrap(SetTextStyle(BgColorCfg(Some((n - 40) as u8)))),
                    49              => wrap(SetTextStyle(BgColorCfg(None))),
                    n @ 90...97     => wrap(SetTextStyle(FgColorCfg(Some((n - 82) as u8)))),
                    n @ 100...107   => wrap(SetTextStyle(BgColorCfg(Some((n - 92) as u8)))),
                    _               => None
                })
                }
            },
            (b'm', b'>', 0)     => unimplemented!(),
            (b'n', 0, 0)        => match self.arg(0,5) {
                5   => wrap(StaticResponse("\x1b[0n")),
                6   => wrap(ReportPosition(Code::ANSI)),
                _   => None
            },
            (b'n', b'>', 0)     => unimplemented!(),
            (b'n', b'?', 0)     => unimplemented!(),
            (b'p', 0, b'!')     => unimplemented!(),
            (b'p', 0, b'$')     => unimplemented!(),
            (b'p', 0, b'"')     => wrap(NoFeature(self.csi_code())),
            (b'p', b'>', 0)     => unimplemented!(),
            (b'p', b'?', b'$')  => unimplemented!(),
            (b'q', 0, 0)        => wrap(NoFeature(self.csi_code())),
            (b'q', 0, b' ')     => match self.arg(0,1) {
                0 | 1   => unimplemented!(),
                2       => unimplemented!(),
                3       => unimplemented!(),
                4       => unimplemented!(),
                5       => unimplemented!(),
                6       => unimplemented!(),
                _       => None,
            },
            (b'q', 0, b'"')     => wrap(NoFeature(self.csi_code())),
            (b'r', 0, 0)        => wrap(NoFeature(self.csi_code())),
            (b'r', 0, b'$')     => {
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
            (b'r', b'?', 0)     => unimplemented!(),
            (b's', 0, 0)        => unimplemented!(), //left and right margins
            (b's', b'?', 0)     => unimplemented!(),
            (b't', 0, 0)        => unimplemented!(), //window manipulation
            (b't', 0, b' ')     => wrap(NoFeature(self.csi_code())),
            (b't', 0, b'$')     => unimplemented!(), // DECRARA
            (b't', b'>', 0)     => wrap(NoFeature(self.csi_code())),
            (b'u', 0, 0)        => unimplemented!(), // Restore cursor?
            (b'u', 0, b' ')     => wrap(NoFeature(self.csi_code())),
            (b'v', 0, b'$')     => unimplemented!(), // Copy an area
            (b'w', 0, b'\'')    => wrap(NoFeature(self.csi_code())),
            (b'x', 0, 0)        => unimplemented!(),
            (b'x', 0, b'*')     => unimplemented!(),
            (b'x', 0, b'$')     => wrap(NoFeature(self.csi_code())),
            (b'y', 0, b'*')     => unimplemented!(),
            (b'z', 0, b'$')     => unimplemented!(), // erase rectangular area
            (b'z', 0, b'\'')    => wrap(NoFeature(self.csi_code())),
            (b'{', 0, b'\'')    => unimplemented!(),
            (b'{', 0, b'$')     => wrap(NoFeature(self.csi_code())),
            (b'|', 0, b'\'')    => wrap(NoFeature(self.csi_code())),
            (b'}', 0, b'\'')    => unimplemented!(), 
            (b'~', 0, b'\'')    => unimplemented!(), 
            _                   => None
        }
    }

    pub fn dcs(&self, strarg: &str) -> Option<Box<Command>> {
        match (self.private_mode, self.preterminal) {
            (b'|', 0)       => unimplemented!(),
            (b'$', b'q')    => unimplemented!(),
            (b'+', b'p')    => unimplemented!(),
            (b'+', b'q')    => unimplemented!(),
            _               => unreachable!(),
        }
    }

    pub fn osc(&self, strarg: &str) -> Option<Box<Command>> {
        match self.arg(0, 0) {
            0...2   => wrap(SetTitle(RefCell::new(Some(String::from(strarg))))),
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

    fn csi_code(&self) -> String {
        let args = self.args.iter().map(ToString::to_string).collect::<Vec<_>>().join(";");
        format!("^[[{}{}{}{}", self.private_mode as char, args,
                self.preterminal as char, self.terminal as char)
    }

}

fn wrap<T: Command>(cmd: T) -> Option<Box<Command>> {
    Some(Box::new(cmd) as Box<Command>)
}
