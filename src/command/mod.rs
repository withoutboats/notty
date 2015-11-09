use std::fs::File;
use std::io::Write;

use command::prelude::*;
use output::AnsiCode;

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
pub use self::meta::{PushBuffer, PopBuffer, Bell, SetTitle, SetInputMode};
pub use self::movement::{Move, ScrollScreen};
pub use self::put::Put;
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

pub trait Command: Send + 'static {
    fn apply(&self, &mut Screen, &Sender<InputEvent>);
    fn repr(&self) -> String;
}

pub struct CommandSeries(pub Vec<Box<Command>>);

impl Command for CommandSeries {
    fn apply(&self, screen: &mut Screen, tx: &Sender<InputEvent>) {
        for cmd in &self.0 {
            cmd.apply(screen, tx);
        }
    }
    fn repr(&self) -> String {
        String::from("SERIES: ") + &self.0.iter().map(|c| c.repr()).collect::<Vec<_>>().join("; ")
    }
}

pub struct NoFeature(pub String);

impl Command for NoFeature {
    fn apply(&self, _: &mut Screen, _: &Sender<InputEvent>) {
        if let Ok(mut file) = File::open(::cfg::LOGFILE) {
            let _ = write!(file, "{}", self.repr());
        }
    }
    fn repr(&self) -> String {
        format!("NO FEATURE: {}", self.0)
    }
}
