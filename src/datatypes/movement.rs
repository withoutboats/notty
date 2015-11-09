use cfg;
use datatypes::{Coords, Direction};

use self::Movement::*;

/// Represents a manner in which the cursor can be moved.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Movement {
    Position(Coords),
    Up(u32),
    Down(u32),
    Left(u32),
    Right(u32),
    PreviousLine(u32),
    NextLine(u32),
    LeftTab(u32),
    RightTab(u32),
    Column(u32),
    Row(u32),
    ToBeginning,
    ToEnd,
    UpIndex(u32),
    DownIndex(u32),
    LeftIndex(u32),
    RightIndex(u32),
    UpToEdge,
    DownToEdge,
    LeftToEdge,
    RightToEdge,
}

impl Movement {

    /// Given a movement, set the magnitude to `n`. Note that for directional keys, a magnitude of
    /// 0 is equivalent to setting it to the ToEdge form.
    pub fn magnitude(&self, n: u32) -> Movement {
        if n == 0 {
            match *self {
                Column(_)               => Column(0),
                Row(_)                  => Row(0),
                Up(_) | UpToEdge        => UpToEdge,
                Down(_) | DownToEdge    => DownToEdge,
                Left(_) | LeftToEdge    => LeftToEdge,
                Right(_) | RightToEdge  => RightToEdge,
                _                       => *self
            }
        } else {
            match *self {
                Column(_)               => Column(n),
                Row(_)                  => Row(n),
                Up(_) | UpToEdge        => Up(n),
                Down(_) | DownToEdge    => Down(n),
                Left(_) | LeftToEdge    => Left(n),
                Right(_) | RightToEdge  => Right(n),
                PreviousLine(_)         => PreviousLine(n),
                NextLine(_)             => NextLine(n),
                UpIndex(_)              => UpIndex(n),
                DownIndex(_)            => DownIndex(n),
                LeftIndex(_)            => LeftIndex(n),
                RightIndex(_)           => RightIndex(n),
                LeftTab(_)              => LeftTab(n),
                RightTab(_)             => RightTab(n),
                _                       => *self
            }
        }
    }

    /// Convert this movement to a direction and a magnitude, or None if it cannot be represented
    /// as a direction and a magnitude.
    pub fn as_direction(&self) -> Option<(u32, Direction)> {
        match *self {
            Up(n) | UpIndex(n) | PreviousLine(n)    => Some((n, Direction::Up)),
            Down(n) | DownIndex(n) | NextLine(n)    => Some((n, Direction::Down)),
            Left(n) | LeftIndex(n)                  => Some((n, Direction::Left)),
            Right(n) | RightIndex(n)                => Some((n, Direction::Right)),
            LeftTab(n)                              => Some((n * cfg::TAB_STOP, Direction::Left)),
            RightTab(n)                             => Some((n * cfg::TAB_STOP, Direction::Right)),
            _                                       => None,
        }
    }

    /// Returns true if this motion can cause the screen to scroll.
    pub fn scrolls(&self) -> bool {
        match *self {
            UpIndex(_)
                | DownIndex(_)
                | LeftIndex(_)
                | RightIndex(_)
                | PreviousLine(_)
                | NextLine(_)
                => true,
           _    => false,
        }
    }

}
