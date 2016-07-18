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
use std::io;

use terminal::Terminal;
use Command;

mod erase;
mod input;
mod meta;
mod movement;
mod panel;
mod put;
mod respond;
mod style;
mod tooltip;

pub use notty_encoding::cmds::{
    Erase, RemoveChars, RemoveRows, InsertBlank, InsertRows,
    PushPanel, PopPanel, SplitPanel, UnsplitPanel, AdjustPanelSplit,
    RotateSectionDown, RotateSectionUp, SwitchActiveSection,
    SetInputMode,
    Move, ScrollScreen,
    SetCursorStyle, DefaultCursorStyle,
    SetTextStyle, DefaultTextStyle,
    SetStyleInArea, DefaultStyleInArea,
};

pub use self::input::{KeyPress, KeyRelease, Paste};
pub use self::meta::{SetTitle, Bell};
pub use self::put::{Put, PutAt};
pub use self::respond::{StaticResponse, ReportPosition};
pub use self::tooltip::{AddToolTip, RemoveToolTip, AddDropDown};

mod prelude {
    pub use std::io;
    pub use terminal::Terminal;
    pub use super::CommandTrait as Command;
}

pub trait CommandTrait: Send + 'static {
    fn apply(&self, &mut Terminal) -> io::Result<()>;
    #[cfg(any(test, debug_assertions))]
    fn repr(&self) -> String;
}

pub struct CommandSeries(pub Vec<Command>);

impl CommandTrait for CommandSeries {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        for cmd in &self.0 {
            try!(terminal.apply(cmd));
        }
        Ok(())
    }

    #[cfg(any(test, debug_assertions))]
    fn repr(&self) -> String {
        String::from("SERIES: ") + &self.0.iter().map(|c| c.inner.repr())
                                          .collect::<Vec<_>>().join("; ")
    }
}

pub struct NoFeature(pub String);

impl CommandTrait for NoFeature {
    fn apply(&self, _: &mut Terminal) -> io::Result<()> {
        Ok(())
    }

    #[cfg(any(test, debug_assertions))]
    fn repr(&self) -> String {
        format!("NO FEATURE: {}", self.0)
    }
}
