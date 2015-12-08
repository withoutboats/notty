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
use std::io;

use command::*;
use datatypes::args::*;
use grapheme_tables as gr;

mod ansi;
mod notty;

use self::ansi::AnsiData;
use self::notty::NottyData;
use self::State::*;

/// The `Output` struct processes data written to the terminal from the controlling process,
/// parsing it into structured commands. It is implemented as an `Iterator`.
pub struct Output<R: io::BufRead> {
    tty: io::Chars<R>,
    state: State,
    ansi: AnsiData,
    notty: NottyData,
}

impl<R: io::BufRead> Output<R> {

    /// Create a new output processor wrapping a buffered read interface to the tty.
    pub fn new(tty: R) -> Output<R> {
        Output {
            tty: tty.chars(),
            state: Character,
            ansi: AnsiData::default(),
            notty: NottyData::default(),
        }
    }

    fn character(&mut self, ch: char) -> (State, Option<Box<Command>>) {
        use grapheme_tables::GraphemeCat::*;
        match gr::grapheme_category(ch) {
            GC_Any                      => (Character, wrap(Put::new_char(ch))),
            GC_Control                  => match ch {
                '\x07'      => (Character, wrap(Bell)),
                '\x08'      => (Character, wrap(Move::new(To(Left, 1, true)))),
                '\t'        => (Character, wrap(Move::new(Tab(Right, 1, true)))),
                '\n'        => (Character, wrap(Move::new(NextLine(1)))),
                '\r'        => (Character, wrap(Move::new(ToEdge(Left)))),
                '\x1b'      => (EscCode, None),
                '\x7f'      => (Character, wrap(Erase::new(CursorCell))),
                '\u{90}'    => (DcsCode, None),
                '\u{9b}'    => (CsiCode, None),
                '\u{9d}'    => (OscCode, None),
                '\u{9e}'    => (PrivMsg, None),
                '\u{9f}'    => (ApcCode, None),
                _           => (Character, None),
            },
            GC_Extend | GC_SpacingMark  => (Character, wrap(Put::new_extension(ch))),
            _                           => unimplemented!(),
        }
    }

    fn esc_code(&mut self, ch: char) -> (State, Option<Box<Command>>) {
        match ch {
            ' ' => {
                static IGNORE: &'static [char] = &['F', 'G', 'L', 'N'];
                (Ignore(IGNORE), None)
            }
            '#' => {
                static IGNORE: &'static [char] = &['3', '4', '5', '6', '8'];
                (Ignore(IGNORE), None)
            }
            '%' => {
                static IGNORE: &'static [char] = &['@', 'G'];
                (Ignore(IGNORE), None)
            }
            '('...'/'   => {
                static IGNORE: &'static [char] = &[
                    '0', '<', '>', '%', 'A', 'B', '4', 'C', '5', 'R', 'f', 'Q', '9', 'K', 'Y',
                    '`', 'E', '6', 'Z', 'H', '7', '=',
                ];
                (Ignore(IGNORE), None)
            }
            'E' => (Character, wrap(Move::new(NextLine(1)))),
            'P' => (DcsCode, None),
            '[' => (CsiCode, None),
            ']' => (OscCode, None),
            '^' => (PrivMsg, None),
            '_' => (ApcCode, None),
            _   => (Character, wrap(NoFeature(ch.to_string()))),
        }
    }

    fn csi_code(&mut self, ch: char) -> (State, Option<Box<Command>>) {
        static CSI_PRIVATE_MODES:   &'static [char] = &['>', '?'];
        static CSI_PRETERMINALS:    &'static [char] = &[' ', '!', '"', '$', '\'', '*'];
        static CSI_TERMINALS:       &'static [char] = &[
            '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'P', 'S',
            'T', 'X', 'Z', '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'l', 'm', 'n',
            'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '{', '|', '}', '~',
        ];

        // Private mode.
        if self.ansi.private_mode == '\0' && self.ansi.args.len() == 0 &&
            CSI_PRIVATE_MODES.contains(&ch) {
                self.ansi.private_mode = ch;
                (CsiCode, None)
        }
        // Digit.
        else if self.ansi.preterminal == '\0' && ch.is_digit(10) {
            self.ansi.arg_buf.push(ch);
            (CsiCode, None)
        }
        // Arg separator.
        else if self.ansi.preterminal == '\0' && ch == ';' {
            let n = u32::from_str_radix(&self.ansi.arg_buf, 10).unwrap();
            self.ansi.args.push(n);
            self.ansi.arg_buf.clear();
            (CsiCode, None)
        }
        // Preterminal.
        else if self.ansi.preterminal == '\0' && CSI_PRETERMINALS.contains(&ch) {
            if self.ansi.arg_buf.len() > 0 {
                let n = u32::from_str_radix(&self.ansi.arg_buf, 10).unwrap();
                self.ansi.args.push(n);
                self.ansi.arg_buf.clear();
            }
            self.ansi.preterminal = ch;
            (CsiCode, None)
        }
        // Terminal.
        else if CSI_TERMINALS.contains(&ch) {
            if self.ansi.arg_buf.len() > 0 {
                let n = u32::from_str_radix(&self.ansi.arg_buf, 10).unwrap();
                self.ansi.args.push(n);
                self.ansi.arg_buf.clear();
            }
            let ret = (Character, self.ansi.csi(ch));
            self.ansi.clear();
            ret
        }
        // Invalid.
        else {
            self.ansi.clear();
            (Character, None)
        }
    }

    #[allow(unused)]
    fn dcs_code(&mut self, ch: char) -> (State, Option<Box<Command>>) {
        unimplemented!()
    }

    fn osc_code(&mut self, ch: char) -> (State, Option<Box<Command>>) {
        if ch.is_digit(10) && self.ansi.private_mode == '\0' {
            self.ansi.arg_buf.push(ch);
            (OscCode, None)
        }
        else if ch == ';' && self.ansi.private_mode == '\0' {
            let n = u32::from_str_radix(&self.ansi.arg_buf, 10).unwrap();
            self.ansi.args.push(n);
            self.ansi.arg_buf.clear();
            self.ansi.private_mode = ';';
            (OscCode, None)
        }
        else if ch == '\x1b' && self.ansi.preterminal == '\0' {
            self.ansi.preterminal == '\x1b';
            (OscCode, None)
        }
        else if ch == '\u{9c}' || ch == '\x07' || (ch == '\\' && self.ansi.preterminal == '\x1b') {
            let ret = (Character, self.ansi.osc());
            self.ansi.clear();
            ret
        }
        else if self.ansi.private_mode == ';' {
            if self.ansi.preterminal == '\x1b' {
                self.ansi.arg_buf.push('\x1b');
                self.ansi.preterminal = '\0';
            }
            self.ansi.arg_buf.push(ch);
            (OscCode, None)
        }
        else {
            self.ansi.clear();
            self.character(ch)
        }
    }

    fn apc_code(&mut self, ch: char) -> (State, Option<Box<Command>>) {
        match ch {
            '[' => (NottyCode, None),
            _   => self.privacy_message(ch),
        }
    }

    fn privacy_message(&mut self, ch: char) -> (State, Option<Box<Command>>) {
        match (self.ansi.preterminal, ch) {
            ('\0', '\x1b')                                  => {
                self.ansi.preterminal = ch;
                (PrivMsg, None)
            }
            (_, '\u{9c}') | (_, '\x07') | ('\x1b', '\\')    => (Character, None),
            ('\0', _)                                       => (PrivMsg, None),
            (_, _)                                          => self.character(ch),
        }
    }

    fn notty_code(&mut self, ch: char) -> (State, Option<Box<Command>>) {
        if ch.is_digit(16) || ch == ';' || ch == '.' {
            self.notty.args.push(ch);
            (NottyCode, None)
        }
        else if ch == '#' {
            (NottyAttach, None)
        }
        else if ch == '\u{9c}' {
            let ret = (Character, self.notty.parse());
            self.notty.clear();
            ret
        }
        else {
            self.notty.clear();
            (Character, None)
        }
    }

}

impl<R: io::BufRead> Iterator for super::Output<R> {
    type Item = io::Result<Box<Command>>;
    fn next(&mut self) -> Option<io::Result<Box<Command>>> {
        loop {
            match self.tty.next() {
                Some(Ok(ch))                            => {
                    let (state, cmd) = match self.state {
                        Character       => self.character(ch),
                        EscCode         => self.esc_code(ch),
                        CsiCode         => self.csi_code(ch),
                        DcsCode         => self.dcs_code(ch),
                        OscCode         => self.osc_code(ch),
                        ApcCode         => self.apc_code(ch),
                        PrivMsg         => self.privacy_message(ch),
                        NottyCode       => self.notty_code(ch),
                        NottyAttach     => {
                            match self.notty.attachments.append(ch) {
                                Some(true)  => (Character, self.notty.parse()),
                                Some(false) => (Character, None),
                                None        => (NottyAttach, None),
                            }
                        }
                        Ignore(chars)   => {
                            if chars.contains(&ch) { continue }
                            else { self.character(ch) }
                        }
                    };
                    self.state = state;
                    match cmd {
                        Some(cmd)   => return Some(Ok(cmd)),
                        None        => continue
                    }
                },
                Some(Err(io::CharsError::NotUtf8))      => continue,
                Some(Err(io::CharsError::Other(err)))   => return Some(Err(err)),
                None                                    => return None,
            }
        }
    }
}

enum State {
    Character,
    EscCode,
    CsiCode,
    #[allow(dead_code)]
    DcsCode,
    OscCode,
    ApcCode,
    PrivMsg,
    NottyCode,
    NottyAttach,
    Ignore(&'static [char]),
}

fn wrap<T: Command>(cmd: T) -> Option<Box<Command>> {
    Some(Box::new(cmd) as Box<Command>)
}

#[cfg(test)]
mod tests {

    use std::io::BufReader;

    use command::*;
    use super::*;

    fn setup(data: &[u8]) -> Output<BufReader<&[u8]>> {
        Output::new(BufReader::new(data))
    }

    #[test]
    fn graphemes() {
        let mut output = setup("E\u{301}\u{1f4a9}E".as_bytes());
        assert_eq!(&output.next().unwrap().unwrap().repr(), "E");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "\u{301}");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "\u{1f4a9}");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "E");
    }

    #[test]
    fn ctrl_codes() {
        let mut output = setup(b"AB\x07C\n");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "A");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "B");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "BELL");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "C");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "MOVE NEXT LINE 1");
    }

    #[test]
    fn csi_code() {
        let mut output = setup(b"\x1b[7;7HB\x1b[7A\x1b[$rA\x1b[?12h");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "MOVE TO 6,6");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "B");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "MOVE UP 7");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "DEFAULT STYLE IN AREA");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "A");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "SERIES: SET CURSOR STYLE");
    }

    #[test]
    fn osc_code() {
        let mut output = setup(b"A\x1b]0;Hello, world!\x07B");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "A");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "SET TITLE");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "B");
    }

    #[test]
    fn notty_code() {
        let mut output = setup("A\x1b_[30;8.ff.ff.ff\u{9c}\x1b_[19;1;2\u{9c}B".as_bytes());
        assert_eq!(&output.next().unwrap().unwrap().repr(), "A");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "SET TEXT STYLE");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "SCROLL SCREEN");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "B");
    }

}
