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
//! The datatypes module defines the abstract datatypes used by other components of notty.
//! 
//! The types in this module are intended to be passed between modules. As a design restriction,
//! any methods on any type in this submodule are required to take the receiver immutably.
use std::cmp;

use mime::Mime;

mod iter;
mod key;

pub use self::iter::CoordsIter;
pub use self::key::Key;

pub use notty_encoding::args::*;

pub mod args {
    pub use super::{
        Area,
        BufferSettings,
        Coords,
        Color,
        Direction,
        EchoSettings,
        InputSettings,
        MediaAlignment,
        MediaPosition,
        Movement,
        Region,
        Style,
    };
    pub use super::Area::*;
    pub use super::Direction::*;
    pub use super::InputSettings::*;
    pub use super::MediaAlignment::*;
    pub use super::MediaPosition::*;
    pub use super::Movement::*;
    pub use super::Style::*;
    pub use notty_encoding::args::Argument;
}

/// Data that could be placed in a character cell.
#[derive(Clone)]
pub enum CellData {
    /// A single unicode code point.
    Char(char), 
    /// An extension code point such as U+301. Normally, writing this to the screen does not
    /// overwrite a cell, but instead applies it to the char in the cell.
    ExtensionChar(char),
    /// Non-character media data, with a mime type, positioning and size info.
    Image {
        pos: MediaPosition,
        width: u32,
        height: u32,
        data: Vec<u8>,
        mime: Mime,
    }
}

/// A kind of escape code format (used for structuring response strings).
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Code {
    ANSI,
    Notty,
}

/// Calculate the movement from one coordinate to another within a region.
pub fn move_within(Coords {x, y}: Coords, movement: Movement, region: Region, tab: u32) -> Coords {
    use self::Movement::*;
    use self::Direction::*;
    match movement {
        Position(coords)    => region.xy_within(coords),
        Column(n)           => Coords {x: region.x_within(n), y: y},
        Row(n)              => Coords {x: x, y: region.y_within(n)},
        ToEdge(Up)          => Coords {x: x, y: region.top},
        ToEdge(Down)        => Coords {x: x, y: region.bottom - 1},
        ToEdge(Left)        => Coords {x: region.left, y: y},
        ToEdge(Right)       => Coords {x: region.right - 1, y: y},
        ToBeginning         => Coords {x: region.left, y: region.top},
        ToEnd               => Coords {x: region.right - 1, y: region.bottom - 1},
        To(Up, n, true) if region.top + n > y       => {
            let x = x.saturating_sub((region.top + n - y) / (region.bottom - region.top) + 1);
            let y = region.bottom - (region.top + n - y) % (region.bottom - region.top);
            if x < region.left {
                Coords { x: region.left, y: region.top }
            } else {
                Coords { x: x, y: y }
            }
        }
        To(Down, n, true) if y + n >= region.bottom  => {
            let x = x + (y + n - region.bottom) / (region.bottom - region.top) + 1;
            let y = region.top + (y + n - region.bottom) % (region.bottom - region.top);
            if x >= region.right {
                Coords { x: region.right - 1, y: region.bottom - 1 }
            } else {
                Coords { x: x, y: y }
            }
        }
        To(Left, n, true) if region.left + n > x    => {
            let y = y.saturating_sub((region.left + n - x) / (region.right - region.left) + 1);
            let x = region.right - (region.left + n - x) % (region.right - region.left);
            if y < region.top {
                Coords { x: region.left, y: region.top }
            } else {
                Coords { x: x, y: y }
            }
        }
        To(Right, n, true) if x + n >= region.right  => {
            let y = y + (x + n - region.right) / (region.right - region.left) + 1;
            let x = region.left + (x + n - region.right) % (region.right - region.left);
            if y >= region.bottom {
                Coords { x: region.right - 1, y: region.bottom - 1 }
            } else {
                Coords { x: x, y: y }
            }
        }
        To(Up, n, _) | IndexTo(Up, n)         => {
            Coords {x: x, y: cmp::max(region.top, y.saturating_sub(n))}
        }
        To(Down, n, _) | IndexTo(Down, n)     => {
            Coords {x: x, y: cmp::min(y.saturating_add(n), region.bottom - 1)}
        }
        To(Left, n, _) | IndexTo(Left, n)     => {
            Coords {x: cmp::max(region.left, x.saturating_sub(n)), y: y}
        }
        To(Right, n, _) | IndexTo(Right, n)   => {
            Coords {x: cmp::min(x.saturating_add(n), region.right - 1), y: y}
        }
        Tab(Left, n, true) if region.left + n > x => {
            unimplemented!()
        }
        Tab(Right, n, true) if x + n >= region.right => {
            unimplemented!()
        }
        Tab(Left, n, _)                 => {
            let tab = ((x / tab).saturating_sub(n)) * tab;
            Coords {x: cmp::max(tab, region.left), y: y}
        }
        Tab(Right, n, _)                => {
            let tab = ((x / tab) + n) * tab;
            Coords {x: cmp::min(tab, region.right - 1), y: y}
        }
        Tab(..)                             => unimplemented!(),
        PreviousLine(n)                     => {
            Coords {x: 0, y: cmp::max(y.saturating_sub(n), region.top)}
        }
        NextLine(n)                         => {
            Coords {x: 0, y: cmp::min(y.saturating_add(n), region.bottom - 1)}
        }
    }
}
