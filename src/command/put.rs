use std::cell::RefCell;

use command::prelude::*;
use datatypes::CellData;

#[derive(Eq, PartialEq)]
pub struct Put(RefCell<Option<CellData>>);

impl Put {
    pub fn new_char(ch: char) -> Put {
        Put(RefCell::new(Some(CellData::Char(ch))))
    }
    pub fn new_extension(ch: char) -> Put {
        Put(RefCell::new(Some(CellData::ExtensionChar(ch))))
    }
    pub fn new_grapheme(ch: String) -> Put {
        Put(RefCell::new(Some(CellData::Grapheme(ch))))
    }
}

impl Command for Put {
    fn apply(&self, screen: &mut Screen, _: &mut FnMut(InputEvent)) {
        if let Some(data) = self.0.borrow_mut().take() {
            screen.write(data)
        }
    }
    fn repr(&self) -> String {
        match *self.0.borrow() {
            Some(CellData::Char(c)) | Some(CellData::ExtensionChar(c))
                                            => c.to_string(),
            Some(CellData::Grapheme(ref c)) => c.clone(),
            _                               => String::from("PUT"),
        }
    }
}
