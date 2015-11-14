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
use std::cmp::Ordering;

use datatypes::{Coords, Direction};
use datatypes::Direction::*;

use self::Movement::*;

/// Represents a manner in which the cursor can be moved.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Movement {
    Position(Coords),
    To(Direction, u32, bool),
    ToEdge(Direction),
    IndexTo(Direction, u32),
    Tab(Direction, u32, bool),
    Column(u32),
    Row(u32),
    PreviousLine(u32),
    NextLine(u32),
    ToBeginning,
    ToEnd,
}

impl Movement {

    /// Returns the direction the cursor would travel in on taking this movement.
    pub fn direction(&self, cursor: Coords) -> Direction {
        match *self {
            To(d, _, _) | ToEdge(d) | IndexTo(d, _) | Tab(d, _, _)  => d,
            ToBeginning                                             => Left,
            ToEnd                                                   => Right,
            PreviousLine(_)                                         => Up,
            NextLine(_)                                             => Down,
            Column(n) if n < cursor.x                               => Left,
            Column(_)                                               => Right,
            Row(n) if n < cursor.y                                  => Up,
            Row(_)                                                  => Down,
            Position(coords)                                        => {
                match coords.y.cmp(&cursor.y) {
                    Ordering::Less                                  => Left,
                    Ordering::Equal if coords.x < cursor.x          => Left,
                    Ordering::Equal                                 => Right,
                    Ordering::Greater                               => Right,
                }
            }
        }
    }

    /// Returns true if this motion can cause the screen to scroll.
    pub fn scrolls(&self) -> bool {
        match *self {
            IndexTo(..) | PreviousLine(_) | NextLine(_) => true,
           _                                            => false,
        }
    }

}
