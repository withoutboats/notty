use datatypes::{Coords, Region, Movement};

use self::Area::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Area {
    CursorCell,
    CursorRow,
    CursorColumn,
    CursorTo(Movement),
    CursorBound(Coords),
    WholeScreen,
    Bound(Region),
    Rows(u32, u32),
    Columns(u32, u32),
    BelowCursor(bool),
}
