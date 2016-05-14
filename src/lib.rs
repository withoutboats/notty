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
#![feature(io, iter_arith)]

extern crate base64;
extern crate mime;
extern crate notty_encoding;
extern crate unicode_width;

mod command;
pub mod datatypes;
mod grapheme_tables;
mod output;
pub mod terminal;

pub use output::Output;

use command::{KeyPress, KeyRelease, CommandTrait};
use datatypes::Key;

/// A command to be applied to the terminal.
///
/// `Command` is an opaque wrapper type around the various commands, which does not allow for
/// introspection or reflection. The `Output` iterator generates `Command` objects transmitted from
/// the output of the controlling process. User input is also represented as a command, and
/// constructors exist for creating `Command` objects with the correct internal representation for
/// different kinds of user input.
///
/// Commands are passed to the `Terminal` with the `apply` method.
pub struct Command {
    inner: Box<CommandTrait>,
}

impl Command {
    /// Create a command representing a key press event.
    pub fn key_press(key: Key) -> Command {
        Command {
            inner: Box::new(KeyPress(key)) as Box<CommandTrait>,
        }
    }
    /// Create a command representing a key release event.
    pub fn key_release(key: Key) -> Command {
        Command {
            inner: Box::new(KeyRelease(key)) as Box<CommandTrait>,
        }
    }
}

use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT};
/// The amount of scrollback to save in terminal grids which save their scrollback. If you do not
/// set this to a non-zero value, it will be set to 512 when the terminal is initialized.
pub static SCROLLBACK:  AtomicUsize = ATOMIC_USIZE_INIT;
/// The distance between each tab stop. If you do not set this to a non-zero value, it will be set
/// to 4 when the terminal is initialized.
pub static TAB_STOP:    AtomicUsize = ATOMIC_USIZE_INIT;
