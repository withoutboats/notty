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
use std::fs::File;
use std::io::Write;

use command::prelude::*;

mod echo;
mod erase;
mod input;
mod meta;
mod movement;
mod put;
mod respond;
mod style;
mod tooltip;

pub use notty_encoding::cmds::{
    SetBufferMode, SetEchoMode,
    Erase, RemoveChars, RemoveRows, InsertBlank, InsertRows,
    PushBuffer, PopBuffer, SetInputMode,
    Move, ScrollScreen,
    SetCursorStyle, DefaultCursorStyle,
    SetTextStyle, DefaultTextStyle,
    SetStyleInArea, DefaultStyleInArea,
};

pub use self::input::{KeyPress, KeyRelease};
pub use self::meta::{SetTitle, Bell};
pub use self::put::{Put, PutAt};
pub use self::respond::{StaticResponse, ReportPosition};
pub use self::tooltip::{AddToolTip, RemoveToolTip, AddDropDown};

mod prelude {
    pub use std::io;
    pub use terminal::Terminal;
    pub use super::Command;
}

/// A command to be applied to the terminal.
///
/// Dynamically dispatched `Command` objects are generated from the `Output` iterator, which
/// parses the output of the controlling process into applicable `Command` events. Most of the
/// implementers of `Command` are private types within this library, so that the process of
/// application remains opaque to downstream users. The only exported types are those dealing with
/// direc user input.
pub trait Command: Send + 'static {
    /// Apply this command to the terminal.
    fn apply(&self, &mut Terminal) -> io::Result<()>;
    fn repr(&self) -> String;
}

pub struct CommandSeries(pub Vec<Box<Command>>);

impl Command for CommandSeries {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        for cmd in &self.0 {
            try!(cmd.apply(terminal));
        }
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("SERIES: ") + &self.0.iter().map(|c| c.repr()).collect::<Vec<_>>().join("; ")
    }
}

pub struct NoFeature(pub String);

impl Command for NoFeature {
    fn apply(&self, _: &mut Terminal) -> io::Result<()> {
        if let Ok(mut file) = File::open(::cfg::LOGFILE) {
            let _ = write!(file, "{}", self.repr());
        }
        Ok(())
    }
    fn repr(&self) -> String {
        format!("NO FEATURE: {}", self.0)
    }
}
