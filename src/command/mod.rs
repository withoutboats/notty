use std::fs::File;
use std::io::Write;

use command::prelude::*;

mod erase;
mod meta;
mod movement;
mod put;
mod respond;
mod style {
    pub mod cursor;
    pub mod text;
    pub mod textarea;
}

pub use self::erase::{Erase, RemoveChars, RemoveRows, InsertBlank, InsertRows};
pub use self::meta::{AddToolTip, RemoveToolTip, PushBuffer, PopBuffer, Bell, SetTitle, SetInputMode};
pub use self::movement::{Move, ScrollScreen};
pub use self::put::{Put, PutAt};
pub use self::respond::{StaticResponse, ReportPosition};
pub use self::style::cursor::{SetCursorStyle, DefaultCursorStyle};
pub use self::style::text::{SetTextStyle, DefaultTextStyle};
pub use self::style::textarea::{SetStyleInArea, DefaultStyleInArea};

mod prelude {
    pub use std::sync::mpsc::Sender;

    pub use super::Command;
    pub use screen::Screen;
    pub use datatypes::InputEvent;
}

/// A command to be applied to the screen.
///
/// Dynamically dispatched `Command` objects are generated from the `Output` iterator, which
/// parses the output of the controlling process into applicable `Command` events. None of the
/// implementers of `Command` are exported from this library, so that the process of application
/// remains opaque to downstream users.
pub trait Command: Send + 'static {
    /// Apply this command to the screen and input.
    ///
    ///
    /// The first argument to this method is a mutable reference to the `Screen` object; most
    /// commands modify the screen state in some way.
    ///
    /// The second argument to this method is a dynamically dispatched function which takes an
    /// `InputEvent`. This function is intended to send the event to the `Input` processor,
    /// immediately or indirectly (such as through an mpsc channel).
    fn apply(&self, &mut Screen, &mut FnMut(InputEvent));
    fn repr(&self) -> String;
}

pub struct CommandSeries(pub Vec<Box<Command>>);

impl Command for CommandSeries {
    fn apply(&self, screen: &mut Screen, input: &mut FnMut(InputEvent)) {
        for cmd in &self.0 {
            cmd.apply(screen, input);
        }
    }
    fn repr(&self) -> String {
        String::from("SERIES: ") + &self.0.iter().map(|c| c.repr()).collect::<Vec<_>>().join("; ")
    }
}

pub struct NoFeature(pub String);

impl Command for NoFeature {
    fn apply(&self, _: &mut Screen, _: &mut FnMut(InputEvent)) {
        if let Ok(mut file) = File::open(::cfg::LOGFILE) {
            let _ = write!(file, "{}", self.repr());
        }
    }
    fn repr(&self) -> String {
        format!("NO FEATURE: {}", self.0)
    }
}
