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
use std::str::FromStr;

use image::{self, DynamicImage, ImageFormat};
use mime::{Mime, TopLevel, SubLevel};

use command::*;
use datatypes::args::*;

mod attachment;

use self::attachment::Attachments;

#[derive(Default)]
pub struct NottyData {
    pub args: String,
    pub attachments: Attachments,
}

impl NottyData {

    pub fn parse(&self) -> Option<Box<Command>> {
        println!("{}", self.args);
        let mut args = self.args.split(';');
        match u32::decode(args.next(), None) {
            Some(0x14)  => {
                let w = match u32::decode(args.next(), None) { Some(w) => w, None => return None };
                let h = match u32::decode(args.next(), None) { Some(h) => h, None => return None };
                let p = MediaPosition::decode(args.next(), Some(MediaPosition::default())).unwrap();
                if let Some(img) = image(self.attachments.iter()) {
                    wrap(Some(Put::new_image(img, p, w, h)))
                } else { None }
            }
            Some(0x15)  => {
                let coords = Coords::decode(args.next(), Some(Coords {x: 0, y: 0})).unwrap();
                let w = match u32::decode(args.next(), None) { Some(w) => w, None => return None };
                let h = match u32::decode(args.next(), None) { Some(h) => h, None => return None };
                let p = MediaPosition::decode(args.next(), Some(MediaPosition::default())).unwrap();
                if let Some(img) = image(self.attachments.iter()) {
                    wrap(Some(PutAt::new_image(img, p, w, h, coords)))
                } else { None }
            }
            Some(0x18)  => {
                wrap(Movement::decode(args.next(), Some(To(Right, 1, true))).map(Move::new))
            }
            Some(0x19)  => {
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
                self.attachments.iter().next().and_then(|data| String::from_utf8(data).ok())
                .and_then(|title| {
                    wrap(Some(SetTitle(RefCell::new(Some(title)))))
                })
            }
            Some(0x50)  => {
                let coords = Coords::decode(args.next(), None).unwrap();
                self.attachments.iter().next().and_then(|data| String::from_utf8(data).ok())
                .and_then(|string| {
                    wrap(Some(AddToolTip(coords, RefCell::new(Some(String::from(string))))))
                })
            }
            Some(0x51)  => {
                let coords = Coords::decode(args.next(), None).unwrap();
                self.attachments.iter().map(|data| String::from_utf8(data).ok())
                .collect::<Option<_>>().and_then(|data| wrap(Some(AddDropDown {
                    coords: coords,
                    options: RefCell::new(Some(data)),
                })))
            }
            Some(0x54)  => wrap(Coords::decode(args.next(), None).map(RemoveToolTip)),
            Some(0x60)  => wrap(bool::decode(args.next(), Some(false)).map(PushBuffer)),
            Some(0x61)  => wrap(Some(PopBuffer)),
            Some(0x80)  => wrap(InputMode::decode(args.next(), Some(Ansi(false))).map(SetInputMode)),
            Some(0x84)  => wrap(Some(SetBufferMode(BufferSettings::decode(args.next(), None)))),
            Some(0x88)  => wrap(Some(SetEchoMode(EchoSettings::decode(args.next(), None)))),
            _           => None,
        }
    }

    pub fn clear(&mut self) {
        self.args.clear();
        self.attachments.clear();
    }

}

fn image<I: Iterator<Item=Vec<u8>>>(mut attachments: I) -> Option<DynamicImage> {

    let mime = match attachments.next()
        .and_then(|data| String::from_utf8(data).ok())
        .and_then(|string| Mime::from_str(&string).ok()) { Some(m) => m, None => return None };
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

    image::load_from_memory_with_format(&data, fmt).ok()

}

fn wrap<T: Command>(cmd: Option<T>) -> Option<Box<Command>> {
    cmd.map(|cmd| Box::new(cmd) as Box<Command>)
}
