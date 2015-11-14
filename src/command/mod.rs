use std::fs::File;
use std::io::Write;

use command::prelude::*;

mod erase;
mod input;
mod meta;
mod movement;
mod put;
mod respond;
mod style {
    pub mod cursor;
    pub mod text;
    pub mod textarea;
}
mod tooltip;

pub use self::erase::{Erase, RemoveChars, RemoveRows, InsertBlank, InsertRows};
pub use self::input::{KeyPress, KeyRelease};
pub use self::meta::{PushBuffer, PopBuffer, Bell, SetTitle, SetInputMode};
pub use self::movement::{Move, ScrollScreen};
pub use self::put::{Put, PutAt};
pub use self::respond::{StaticResponse, ReportPosition};
pub use self::style::cursor::{SetCursorStyle, DefaultCursorStyle};
pub use self::style::text::{SetTextStyle, DefaultTextStyle};
pub use self::style::textarea::{SetStyleInArea, DefaultStyleInArea};
pub use self::tooltip::{AddToolTip, RemoveToolTip, AddDropDown};

mod prelude {
    pub use std::io;
    pub use terminal::Terminal;
    pub use super::Command;
}

/// A command to be applied to the terminal.
///
/// Dynamically dispatched `Command` objects are generated from the `Output` iterator, which
/// parses the output of the controlling process into applicable `Command` events. None of the
/// implementers of `Command` are exported from this library, so that the process of application
/// remains opaque to downstream users.
pub trait Command: Send + 'static {
    /// Apply this command to the terminal and input.
    ///
    ///
    /// The first argument to this method is a mutable reference to the `Terminal` object; most
    /// commands modify the terminal state in some way.
    ///
    /// The second argument to this method is a dynamically dispatched function which takes an
    /// `InputEvent`. This function is intended to send the event to the `Input` processor,
    /// immediately or indirectly (such as through an mpsc channel).
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
