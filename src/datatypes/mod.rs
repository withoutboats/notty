mod key;
mod movement;
mod region;
mod vector;

pub use self::key::{Key, Modifiers};
pub use self::movement::Movement;
pub use self::region::Region;
pub use self::vector::Vector;

pub mod args {
    pub use super::{Area, Coords, Color, InputMode, Movement, Region, Style};
    pub use super::Area::*;
    pub use super::InputMode::*;
    pub use super::Movement::*;
    pub use super::Style::*;
}

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

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum CellData {
    Char(char),
    ExtensionChar(char),
    Grapheme(String),
    Data(String, Vec<u8>),
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Code {
    ANSI,
    Natty,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Color(pub u8, pub u8, pub u8);

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct Coords {
    pub x: u32,
    pub y: u32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Eq, PartialEq)]
pub enum InputEvent {
    Key(Key),
    Mode(InputMode),
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum InputMode {
    Ansi,
    Application,
    Extended,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Style {
    Underline(u8),
    Bold(bool),
    Italic(bool),
    Blink(bool),
    InvertColors(bool),
    Strikethrough(bool),
    Opacity(u8),
    FgColor(Color),
    FgColorCfg(Option<u8>),
    BgColor(Color),
    BgColorCfg(Option<u8>),
}
