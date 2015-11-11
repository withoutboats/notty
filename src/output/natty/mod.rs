use command::*;
use datatypes::args::*;

mod argument;
mod attachment;

use self::argument::Argument;
use self::attachment::Attachments;

#[derive(Default)]
pub struct NattyCode {
    pub args: String,
    pub attachments: Attachments,
}

impl NattyCode {

    pub fn parse(&self) -> Option<Box<Command>> {
        let mut args = self.args.split(';');
        match u32::decode(args.next(), None) {
            Some(0x10)  => {
                wrap(Movement::decode(args.next(), Some(To(Right, 1, true))).map(Move::new))
            }
            Some(0x11)  => {
                let dir = Direction::decode(args.next(), Some(Down)).unwrap();
                let n = u32::decode(args.next(), Some(1)).unwrap();
                wrap(Some(ScrollScreen::new(dir, n)))
            }
            Some(0x20)  => {
                wrap(Area::decode(args.next(), Some(CursorCell)).map(Erase::new))
            }
            Some(0x21)  => {
                wrap(u32::decode(args.next(), Some(1)).map(RemoveChars::new))
            }
            Some(0x22)  => {
                let n = u32::decode(args.next(), Some(1)).unwrap();
                wrap(bool::decode(args.next(), Some(true)).map(|f| RemoveRows::new(n, f)))
            }
            Some(0x26)  => {
                wrap(u32::decode(args.next(), Some(1)).map(InsertBlank::new))
            }
            Some(0x27)  => {
                let n = u32::decode(args.next(), Some(1)).unwrap();
                wrap(bool::decode(args.next(), Some(true)).map(|f| InsertRows::new(n, f)))
            }
            Some(0x30)  => {
                match Style::decode(args.next(), None) {
                    Some(style) => wrap(Some(SetTextStyle(style))),
                    None        => wrap(Some(DefaultTextStyle)),
                }
            }
            Some(0x31)  => {
                match Style::decode(args.next(), None) {
                    Some(style) => wrap(Some(SetCursorStyle(style))),
                    None        => wrap(Some(DefaultCursorStyle)),
                }
            }
            Some(0x32)  => {
                let area = Area::decode(args.next(), Some(WholeScreen)).unwrap();
                match Style::decode(args.next(), None) {
                    Some(style) => wrap(Some(SetStyleInArea(area, style))),
                    None        => wrap(Some(DefaultStyleInArea(area))),
                }
            }
            _           => None,
        }
    }

    pub fn clear(&mut self) {
        self.args.clear();
        self.attachments.clear();
    }

}

fn wrap<T: Command>(cmd: Option<T>) -> Option<Box<Command>> {
    cmd.map(|cmd| Box::new(cmd) as Box<Command>)
}
