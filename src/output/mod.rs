use std::io;
use std::str;
use std::sync::mpsc::Sender;

use command::*;
use datatypes::args::*;

use self::ansi::AnsiCode;
use self::grapheme_tables as gr;

mod ansi;
mod grapheme_tables;

pub struct Output<R: io::BufRead> {
    tty: R,
    parser: Parser,
}

impl<R: io::BufRead> Output<R> {
    pub fn new(tty: R) -> Output<R> {
        Output {
            tty: tty,
            parser: Parser::default(),
        }
    }

    pub fn run(&mut self, tx: Sender<Box<Command>>) -> io::Result<()> {
        loop {
            if let Some(cmd) = self.next() {
                tx.send(try!(cmd)).unwrap();
            }
        }
    }

}

impl<R: io::BufRead> Iterator for super::Output<R> {
    type Item = io::Result<Box<Command>>;

    fn next(&mut self) -> Option<io::Result<Box<Command>>> {
        let mut offset = 0;
        loop {
            let ret = match self.tty.fill_buf() {
                Ok(buf)     => match self.parser.parse(buf, &mut offset) {
                    Some(cmd)   => Some(Ok(cmd)),
                    None        => continue,
                },
                Err(err)    => return Some(Err(err)),
            };
            self.tty.consume(offset);
            return ret
        }
    }
}

enum Position {
    Grapheme,
    EscCode,
    CsiCode,
    DcsCode,
    OscCode,
    NattyCode,
}

#[derive(Default)]
struct Parser {
    cat: Option<gr::GraphemeCat>,
    ansi: AnsiCode,
    pos: Option<Position>,
    init: usize,
}

impl Parser {
    fn parse(&mut self, buf: &[u8], offset: &mut usize) -> Option<Box<Command>> {
        match self.pos.take() {
            Some(Position::Grapheme)    => self.grapheme(buf, offset),
            Some(Position::EscCode)     => self.esc(buf, offset),
            Some(Position::CsiCode)     => self.csi(buf, offset),
            Some(Position::DcsCode)     => self.dcs(buf, offset),
            Some(Position::OscCode)     => self.osc(buf, offset),
            Some(Position::NattyCode)   => unimplemented!(),
            None                        => {
                self.init = *offset;
                self.grapheme(buf, offset)
            }
        }
    }

    fn grapheme(&mut self, buf: &[u8], offset: &mut usize) -> Option<Box<Command>> {
        use self::grapheme_tables as gr;
        use self::grapheme_tables::GraphemeCat::*;
        use self::grapheme_tables::GraphemeState::*;

        let mut state = Start;

        'grapheme: loop {
            let ch = match code_point(buf, offset) {
                Some(ch)    => ch,
                None        => { self.pos = Some(Position::Grapheme); return None }
            };
            let cat = self.cat.take().unwrap_or_else(|| gr::grapheme_category(ch.char_at(0)));
            state = match (state, cat) {
                (Start, GC_Any)                     => {
                    *offset += ch.len();
                    return wrap(Put::new_char(ch.char_at(0)));
                }
                (Start, GC_Control)                 => {
                    *offset += ch.len();
                    return self.ctrl_code(ch, buf, offset);
                }
                (Start, GC_L)                       => HangulL,
                (Start, GC_LV) | (Start, GC_V)      => HangulLV,
                (Start, GC_LVT) | (Start, GC_T)     => HangulLVT,
                (Start, GC_Regional_Indicator) | (Regional, GC_Regional_Indicator)
                                                    => Regional,
                (Start, GC_Extend) | (Start, GC_SpacingMark)
                                                    => {
                    *offset += ch.len();
                    return wrap(Put::new_extension(ch.char_at(0)));
                }
                (HangulL, GC_L)                     => HangulL,
                (HangulL, GC_LV) | (HangulL, GC_V)  => HangulLV,
                (HangulL, GC_LVT)                   => HangulLVT,
                (HangulLV, GC_V)                    => HangulLV,
                (HangulLV, GC_T)                    => HangulLVT,
                (HangulLVT, GC_T)                   => HangulLVT,
                _                                   => {
                    self.cat = Some(cat);
                    let s = unsafe { str::from_utf8_unchecked(&buf[self.init..*offset]) };
                    return wrap(Put::new_grapheme(String::from(s)));
                }
            };
            *offset += ch.len();
        }
    }

    fn ctrl_code(&mut self, ch: &str, buf: &[u8], offset: &mut usize) -> Option<Box<Command>> {
        match ch {
            "\x07"      => wrap(Bell),
            "\x08"      => wrap(Move::new(To(Left, 1))),
            "\x09"      => wrap(Move::new(Tab(Right, 1))),
            "\n"        => wrap(Move::new(NextLine(1))),
            "\r"        => wrap(Move::new(ToEdge(Left))),
            "\x1b"      => self.esc(buf, offset),
            "\x7f"      => wrap(Erase::new(CursorCell)),
            "\u{90}"    => self.dcs(buf, offset),
            "\u{9b}"    => self.csi(buf, offset),
            "\u{9d}"    => self.osc(buf, offset),
            "\u{9e}" | "\u{9f}" => {
                ansi_str(buf, offset);
                None
            }
            _           => None
        }
    }

    fn esc(&mut self, buf: &[u8], offset: &mut usize) -> Option<Box<Command>> {
        match byte(buf, *offset) {
            Some(b' ')  => { ignore(buf, offset, &[b'F', b'G', b'L', b'N']); None }
            Some(b'#')  => { ignore(buf, offset, &[b'3', b'4', b'5', b'6', b'8']); None }
            Some(b'%')  => { ignore(buf, offset, &[b'@', b'G']); None }
            Some(b'('...b'/') => {
                ignore(buf, offset, &[b'0', b'<', b'>', b'%', b'A', b'B', b'4', b'C', b'5', b'R',
                                      b'f', b'Q', b'9', b'K', b'Y', b'`', b'E', b'6', b'Z', b'H',
                                      b'7', b'=']);
                None
            }
            Some(b'6')  => wrap(NoFeature(String::from("6"))),
            Some(b'7')  => wrap(NoFeature(String::from("7"))),
            Some(b'8')  => wrap(NoFeature(String::from("8"))),
            Some(b'9')  => wrap(NoFeature(String::from("9"))),
            Some(b'D')  => wrap(NoFeature(String::from("D"))),
            Some(b'E')  => { *offset += 1; wrap(Move::new(NextLine(1))) }
            Some(b'H')  => wrap(NoFeature(String::from("H"))),
            Some(b'M')  => wrap(NoFeature(String::from("M"))),
            Some(b'P')  => { *offset += 1; self.dcs(buf, offset) }
            Some(b'Z')  => wrap(NoFeature(String::from("Z"))),
            Some(b'[')  => { *offset += 1; self.csi(buf, offset) }
            Some(b']')  => { *offset += 1; self.osc(buf, offset) }
            Some(b'^') | Some(b'_') => {  ansi_str(buf, offset); None }
            Some(b'c')  => wrap(NoFeature(String::from("c"))),
            Some(b'N'...b'O')
                | Some(b'V'...b'X')
                | Some(b'l'...b'o')
                | Some(b'|'...b'~') => { *offset += 1; None }
            Some(_)     => None,
            None        => { self.pos = Some(Position::EscCode); None }
        }
    }

    fn csi(&mut self, buf: &[u8], offset: &mut usize) -> Option<Box<Command>> {
        // These must be kept sorted!!
        static CSI_PRIVATE_MODES:   &'static [u8]   = &[b'>', b'?'];
        static CSI_PRETERMINALS:    &'static [u8]   = &[b' ', b'!', b'"', b'$', b'\'', b'*'];
        static CSI_TERMINALS:       &'static [u8]   = &[
            b'@', b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M',
            b'P', b'S', b'T', b'X', b'Z', b'`', b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h',
            b'i', b'l', b'm', b'n', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y',
            b'z', b'{', b'|', b'}', b'~',
        ];

        'csi: loop {
            match byte(buf, *offset) {
                Some(ch) if CSI_PRIVATE_MODES.binary_search(&ch).is_ok() => {
                    self.ansi.private_mode = ch;
                    *offset += 1;
                }
                Some(ch) if CSI_PRETERMINALS.binary_search(&ch).is_ok() => {
                    self.ansi.preterminal = ch;
                    *offset += 1;
                }
                Some(ch) if CSI_TERMINALS.binary_search(&ch).is_ok() => {
                    self.ansi.terminal = ch;
                    *offset += 1;
                    break 'csi;
                }
                Some(b'0'...b'9') => {
                    match ansi_num(buf, offset) {
                        Some(n) => self.ansi.args.push(n),
                        None    => {
                            self.pos = Some(Position::CsiCode);
                            return None;
                        }
                    }
                }
                Some(b';')  => {
                    *offset += 1;
                    continue 'csi;
                }
                Some(_)     => return None,
                None        => {
                    self.pos = Some(Position::CsiCode);
                    return None;
                }
            }
        }
        let ret = self.ansi.csi();
        self.ansi.clear();
        ret
    }

    fn dcs(&mut self, buf: &[u8], offset: &mut usize) -> Option<Box<Command>> {
        unimplemented!();
    }

    fn osc(&mut self, buf: &[u8], offset: &mut usize) -> Option<Box<Command>> {
        'osc: loop {
            match byte(buf, *offset) {
                Some(b';')          => {
                    *offset += 1;
                    break 'osc;
                }
                Some(b'0'...b'9')   => {
                    ansi_num(buf, offset).map(|n| self.ansi.args.push(n));
                }
                Some(_)             => return None,
                None                => {
                    self.pos = Some(Position::OscCode);
                    return None;
                }
            }
        }

        match ansi_str(buf, offset) {
            Some(Some(s))   => {
                let ret = self.ansi.osc(s);
                self.ansi.clear();
                ret
            }
            Some(None)      => None,
            None            => { self.pos = Some(Position::OscCode); None }
        }

    }

}

fn byte(buf: &[u8], offset: usize) -> Option<u8> {
    buf.get(offset).map(|&x|x)
}

fn code_point<'a>(buf: &'a [u8], offset: &mut usize) -> Option<&'a str> {
    let width = match byte(buf, *offset) {
        Some(0x00...0x7f)   => 1,
        Some(0xc3...0xdf)   => 2,
        Some(0xe0...0xef)   => 3,
        Some(0xf0...0xf4)   => 4,
        Some(_)             => {
            *offset += 1;
            return None;
        }
        None                => return None,
    };
    match str::from_utf8(&buf[*offset..(*offset + width)]) {
        Ok(s)   => Some(s),
        _       => {
            *offset += 1;
            None
        }
    }
}

fn ansi_str<'a>(buf: &'a [u8], offset: &mut usize) -> Option<Option<&'a str>> {
    let mut offset_tmp = *offset;
    loop {
        match byte(buf, offset_tmp) {
            Some(b'\x07')   => {
                let ret = str::from_utf8(&buf[*offset..offset_tmp]).ok();
                *offset = offset_tmp + 1;
                return Some(ret)
            }
            Some(_)         => offset_tmp += 1,
            None            => return None,
        }
    }
}

fn ansi_num(buf: &[u8], offset: &mut usize) -> Option<u32> {
    let mut offset_tmp = *offset;
    loop {
        match byte(buf, offset_tmp) {
            Some(b'0'...b'9')   => offset_tmp += 1,
            Some(_)             => {
                return str::from_utf8(&buf[*offset..offset_tmp]).ok().and_then(|s| {
                    u32::from_str_radix(s, 10).ok()
                }).map(|n| { *offset = offset_tmp; n })
            }
            None                => {
                *offset -= 1;
                return None
            }
        }
    }
}

fn ignore(buf: &[u8], offset: &mut usize, ignore: &[u8]) {
    if let Some(c) = byte(buf, *offset + 1) {
        if ignore.contains(&c) {
            *offset += 2;
        }
    }
}

fn wrap<T: Command>(cmd: T) -> Option<Box<Command>> {
    Some(Box::new(cmd) as Box<Command>)
}

#[cfg(test)]
mod tests {

    use std::io::{BufRead, BufReader};
    use command::*;
    use super::*;

    fn setup(data: &[u8]) -> Output<BufReader<&[u8]>> {
        Output::new(BufReader::new(data))
    }

    #[test]
    fn graphemes() {
        let mut output = setup("E\u{301}\u{1f4a9}\u{1101}\u{1161}\u{11a8}E".as_bytes());
        assert_eq!(&output.next().unwrap().unwrap().repr(), "E");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "\u{301}");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "\u{1f4a9}");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "\u{1101}\u{1161}\u{11a8}");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "E");
    }

    #[test]
    fn ctrl_codes() {
        let mut output = setup("AB\x07C\n".as_bytes());
        assert_eq!(&output.next().unwrap().unwrap().repr(), "A");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "B");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "BELL");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "C");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "MOVE NEXT LINE 1");
    }

    #[test]
    fn csi_code() {
        let mut output = setup("\x1b[7;7HB\x1b[7A\x1b[$rA\x1b[?12h".as_bytes());
        assert_eq!(&output.next().unwrap().unwrap().repr(), "MOVE TO 6,6");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "B");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "MOVE UP 7");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "DEFAULT STYLE IN AREA");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "A");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "SERIES: SET CURSOR STYLE");
    }

    //#[test]
    fn dcs_code() {
    }

    #[test]
    fn osc_code() {
        let mut output = setup("A\x1b]0;Hello, world!\x07B".as_bytes());
        assert_eq!(&output.next().unwrap().unwrap().repr(), "A");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "SET TITLE");
        assert_eq!(&output.next().unwrap().unwrap().repr(), "B");
    }

}
