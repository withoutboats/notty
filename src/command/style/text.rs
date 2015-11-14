use command::prelude::*;
use datatypes::Style;

#[derive(Copy, Clone)]
pub struct SetTextStyle(pub Style);

impl Command for SetTextStyle {
    fn apply(&self, terminal: &mut Terminal) {
        terminal.set_style(self.0)
    }
    fn repr(&self) -> String {
        String::from("SET TEXT STYLE")
    }
}

#[derive(Copy, Clone)]
pub struct DefaultTextStyle;

impl Command for DefaultTextStyle {
    fn apply(&self, terminal: &mut Terminal) {
        terminal.reset_styles()
    }
    fn repr(&self) -> String {
        String::from("DEFAULT TEXT STYLE")
    }
}
