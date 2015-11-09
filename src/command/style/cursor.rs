use command::prelude::*;
use datatypes::Style;

#[derive(Copy, Clone)]
pub struct SetCursorStyle(pub Style);

impl Command for SetCursorStyle {
    fn apply(&self, screen: &mut Screen, _: &Sender<InputEvent>) {
        screen.set_cursor_style(self.0)
    }
    fn repr(&self) -> String {
        String::from("SET CURSOR STYLE")
    }
}

#[derive(Copy, Clone)]
pub struct DefaultCursorStyle;

impl Command for DefaultCursorStyle {
    fn apply(&self, screen: &mut Screen, _: &Sender<InputEvent>) {
        screen.reset_cursor_styles()
    }
    fn repr(&self) -> String {
        String::from("DEFAULT CURSOR STYLE")
    }
}
