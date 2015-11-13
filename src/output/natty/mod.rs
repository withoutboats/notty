use std::cell::RefCell;
use std::str::{self, FromStr};

use image::{self, DynamicImage, ImageFormat};
use mime::{Mime, TopLevel, SubLevel};

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
            Some(0x04)  => {
                let w = match u32::decode(args.next(), None) { Some(w) => w, None => return None };
                let h = match u32::decode(args.next(), None) { Some(h) => h, None => return None };
                let p = MediaPosition::decode(args.next(), Some(MediaPosition::default())).unwrap();
                if let Some(img) = image(self.attachments.iter()) {
                    wrap(Some(Put::new_image(img, p, w, h)))
                } else { None }
            }
            Some(0x05)  => {
                let coords = Coords::decode(args.next(), Some(Coords {x: 0, y: 0})).unwrap();
                let w = match u32::decode(args.next(), None) { Some(w) => w, None => return None };
                let h = match u32::decode(args.next(), None) { Some(h) => h, None => return None };
                let p = MediaPosition::decode(args.next(), Some(MediaPosition::default())).unwrap();
                if let Some(img) = image(self.attachments.iter()) {
                    wrap(Some(PutAt::new_image(img, p, w, h, coords)))
                } else { None }
            }
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
            Some(0x40)  => {
                self.attachments.iter().next().and_then(|data| str::from_utf8(data).ok())
                .and_then(|title| {
                    wrap(Some(SetTitle(RefCell::new(Some(String::from(title))))))
                })
            }
            Some(0x41)  => wrap(Coords::decode(args.next(), None).map(RemoveToolTip)),
            Some(0x50)  => {
                let coords = Coords::decode(args.next(), None).unwrap();
                self.attachments.iter().next().and_then(|data| str::from_utf8(data).ok())
                .and_then(|string| {
                    wrap(Some(AddToolTip(coords, RefCell::new(Some(String::from(string))))))
                })
            }
            Some(0x60)  => wrap(bool::decode(args.next(), Some(false)).map(PushBuffer)),
            Some(0x61)  => wrap(Some(PopBuffer)),
            Some(0x80)  => wrap(InputMode::decode(args.next(), Some(Ansi)).map(SetInputMode)),
            _           => None,
        }
    }

    pub fn clear(&mut self) {
        self.args.clear();
        self.attachments.clear();
    }

}

fn image<'a, I: Iterator<Item=&'a [u8]>>(mut attachments: I) -> Option<DynamicImage> {

    let mime = match attachments.next()
        .and_then(|data| str::from_utf8(data).ok())
        .and_then(|string| Mime::from_str(string).ok()) { Some(m) => m, None => return None };
    let fmt = match (mime.0, mime.1) {
        (TopLevel::Image, SubLevel::Gif) | (TopLevel::Star, SubLevel::Gif)      => {
            ImageFormat::GIF
        }
        (TopLevel::Image, SubLevel::Jpeg) | (TopLevel::Star, SubLevel::Jpeg)    => {
            ImageFormat::JPEG
        }
        (TopLevel::Image, SubLevel::Png) | (TopLevel::Star, SubLevel::Png)      => {
            ImageFormat::PNG
        }
        _                                                                       => return None
    };

    let data = match attachments.next() { Some(data) => data, None => return None };

    image::load_from_memory_with_format(data, fmt).ok()

}

fn wrap<T: Command>(cmd: Option<T>) -> Option<Box<Command>> {
    cmd.map(|cmd| Box::new(cmd) as Box<Command>)
}
