use command::prelude::*;
use datatypes::Style;

#[derive(Copy, Clone)]
pub struct SetTextStyle(pub Style);

impl Command for SetTextStyle {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.set_style(self.0);
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("SET TEXT STYLE")
    }
}

#[derive(Copy, Clone)]
pub struct DefaultTextStyle;

impl Command for DefaultTextStyle {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.reset_styles();
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("DEFAULT TEXT STYLE")
    }
}
