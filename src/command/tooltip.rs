use std::cell::RefCell;

use command::prelude::*;
use datatypes::Coords;

pub struct AddToolTip(pub Coords, pub RefCell<Option<String>>);

impl Command for AddToolTip {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        if let Some(string) = self.1.borrow_mut().take() {
            terminal.add_tooltip(self.0, string);
        }
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("ADD TOOL TIP")
    }
}

pub struct RemoveToolTip(pub Coords);

impl Command for RemoveToolTip {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.remove_tooltip(self.0);
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("REMOVE TOOL TIP")
    }
}

pub struct AddDropDown {
    pub coords: Coords,
    pub options: RefCell<Option<Vec<String>>>,
}

impl Command for AddDropDown {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        if let Some(options) = self.options.borrow_mut().take() {
            terminal.add_drop_down(self.coords, options);
        }
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("ADD TOOL TIP - DROP DOWN MENU")
    }
}
