use std::cmp::Ordering;

use cfg;
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

    /// Given a movement, set the magnitude to `n`. Note that for directional keys, a magnitude of
    /// 0 is equivalent to setting it to the ToEdge form.
    pub fn magnitude(&self, n: u32) -> Movement {
        if n == 0 {
            match *self {
                Column(_)                   => Column(0),
                Row(_)                      => Row(0),
                To(d, _, _)
                    | IndexTo(d, _)
                    | Tab(d, _, _)          => ToEdge(d),
                _                           => *self
            }
        } else {
            match *self {
                Column(_)                   => Column(n),
                Row(_)                      => Row(n),
                To(d, _, true)              => To(d, n, true),
                To(d, _, false) | ToEdge(d) => To(d, n, false),
                IndexTo(d, _)               => IndexTo(d, n),
                PreviousLine(_)             => PreviousLine(n),
                NextLine(_)                 => NextLine(n),
                Tab(d, _, flag)             => Tab(d, n, flag),
                _                           => *self
            }
        }
    }

    pub fn direction(&self, cursor: Coords) -> Direction {
        match *self {
            To(d, _, _) | ToEdge(d) | IndexTo(d, _) | Tab(d, _, _)  => d,
            ToBeginning | PreviousLine(_)                           => Left,
            ToEnd | NextLine(_)                                     => Right,
            Column(n) if n < cursor.x                               => Left,
            Column(n)                                               => Right,
            Row(n) if n < cursor.y                                  => Up,
            Row(n)                                                  => Down,
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

    pub fn as_direction(&self) -> Option<(u32, Direction)> {
        match *self {
            To(d, n, _) | IndexTo(d, n)             => Some((n, d)),
            Tab(d, n, _)                            => Some((n * cfg::TAB_STOP, d)),
            PreviousLine(n)                         => Some((n, Up)),
            NextLine(n)                             => Some((n, Down)),
            _                                       => None,
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
