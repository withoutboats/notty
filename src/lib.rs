#![feature(str_char, drain)]

extern crate image;
extern crate mime;
extern crate unicode_width;

pub mod cfg;
mod command;
pub mod datatypes;
mod output;
pub mod terminal;

pub use command::{Command, KeyPress, KeyRelease};
pub use output::Output;
