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

use mime::{Mime, SubLevel};

use Command;
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

    pub fn parse(&self) -> Option<Command> {
        let mut args = self.args.split(';');
        match u32::decode(args.next(), None) {
            Some(0x14)  => {
                let w = match u32::decode(args.next(), None) { Some(w) => w, None => return None };
                let h = match u32::decode(args.next(), None) { Some(h) => h, None => return None };
                let p = MediaPosition::decode(args.next(), Some(MediaPosition::default())).unwrap();
                if let Some((mime, data)) = image(self.attachments.iter()) {
                    wrap(Some(Put::new_image(data, mime, p, w, h)))
                } else { None }
            }
            Some(0x15)  => {
                let coords = Coords::decode(args.next(), Some(Coords {x: 0, y: 0})).unwrap();
                let w = match u32::decode(args.next(), None) { Some(w) => w, None => return None };
                let h = match u32::decode(args.next(), None) { Some(h) => h, None => return None };
                let p = MediaPosition::decode(args.next(), Some(MediaPosition::default())).unwrap();
                if let Some((mime, data)) = image(self.attachments.iter()) {
                    wrap(Some(PutAt::new_image(data, mime, p, w, h, coords)))
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
            Some(0x60)  => wrap(Some(PushPanel(u64::decode(args.next(), None)))),
            Some(0x61)  => wrap(Some(PushPanel(u64::decode(args.next(), None)))),
            Some(0x62)  => {
                let l_tag = u64::decode(args.next(), None);
                let r_tag = u64::decode(args.next(), None);
                let kind = SplitKind::decode(args.next(), None);
                let save = SaveGrid::decode(args.next(), Some(SaveGrid::Left));
                let rule = ResizeRule::decode(args.next(), Some(ResizeRule::Percentage));
                let split_tag = u64::decode(args.next(), None);
                match (l_tag, r_tag, kind, save, rule) {
                    (Some(l_t), Some(r_t), Some(k), Some(s), Some(r)) => {
                        wrap(Some(SplitPanel::new(l_t, r_t, k, s, r, split_tag)))
                    }
                    _ => None
                }
            }
            Some(0x63)  => {
                SaveGrid::decode(args.next(), Some(SaveGrid::Left)).and_then(|save| {
                    wrap(Some(UnsplitPanel::new(save, u64::decode(args.next(), None))))
                })
            }
            Some(0x67)  => wrap(u64::decode(args.next(), None).map(SwitchActivePanel)),
            Some(0x80)  => wrap(InputSettings::decode(args.next(), Some(Ansi(false))).map(SetInputMode)),
            _           => None,
        }
    }

    pub fn clear(&mut self) {
        self.args.clear();
        self.attachments.clear();
    }

}

fn image<I: Iterator<Item=Vec<u8>>>(mut attachments: I) -> Option<(Mime, Vec<u8>)> {
    attachments.next().and_then(|data| {
        String::from_utf8(data).ok()
    }).and_then(|string| {
        Mime::from_str(&string).ok()
    }).and_then(|mime| match mime.1 {
        SubLevel::Gif | SubLevel::Jpeg | SubLevel::Png  => Some(mime),
        _                                               => None
    }).and_then(|mime| {
        attachments.next().map(|data| (mime, data))
    })
}

fn wrap<T: CommandTrait>(cmd: Option<T>) -> Option<Command> {
    cmd.map(|cmd| Command { inner: Box::new(cmd) as Box<CommandTrait> })
}
