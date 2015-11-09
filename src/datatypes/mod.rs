mod area;
mod coordinates;
mod key;
mod movement;
mod region;
mod style;
mod vector;

pub use self::area::Area;
pub use self::coordinates::Coords;
pub use self::key::{Key, Modifiers};
pub use self::movement::Movement;
pub use self::region::Region;
pub use self::style::Style;
pub use self::vector::Vector;

pub mod args {
    pub use super::{Area, Coords, Color, InputMode, Movement, Region, Style};
    pub use super::Area::*;
    pub use super::InputMode::*;
    pub use super::Movement::*;
    pub use super::Style::*;
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum CellData {
    Char(char),
    ExtensionChar(char),
    Grapheme(String),
    Data(String, Vec<u8>),
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum InputMode {
    Ansi,
    Application,
    Extended,
}

#[derive(Clone)]
pub enum InputEvent {
    Key(Key),
    Mode(InputMode),
}

pub enum Code { ANSI }

#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

trait Argument: Sized + Copy + Send + Sync {
    fn from_nums<T: IntoIterator<Item=u32>>(T, Option<Self>) -> Option<Self>;
    fn encode(&self) -> String;

    fn decode(s: Option<&str>, default: Option<Self>) -> Option<Self> {
        let iter = s.iter().flat_map(|s| s.split('.')).flat_map(|s| u32::from_str_radix(s, 10));
        Self::from_nums(iter, default)
    }

}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Color(pub u8, pub u8, pub u8);
