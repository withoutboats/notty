use notty_encoding::cmds::{
    SetCursorStyle, DefaultCursorStyle,
    SetTextStyle, DefaultTextStyle,
    SetStyleInArea, DefaultStyleInArea,
};

use command::prelude::*;

impl Command for SetCursorStyle {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.set_cursor_style(self.0);
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("SET CURSOR STYLE")
    }
}

impl Command for DefaultCursorStyle {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.reset_cursor_styles();
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("DEFAULT CURSOR STYLE")
    }
}

impl Command for SetTextStyle {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.set_style(self.0);
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("SET TEXT STYLE")
    }
}

impl Command for DefaultTextStyle {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.reset_styles();
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("DEFAULT TEXT STYLE")
    }
}

impl Command for SetStyleInArea {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.set_style_in_area(self.0, self.1);
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("SET STYLE IN AREA")
    }
}

impl Command for DefaultStyleInArea {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.reset_styles_in_area(self.0);
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("DEFAULT STYLE IN AREA")
    }
}
