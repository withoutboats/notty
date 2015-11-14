use image::DynamicImage;

use std::cell::RefCell;

use command::prelude::*;
use datatypes::{CellData, Coords, MediaPosition};
use datatypes::Movement::Position;

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
    pub fn new_image(data: DynamicImage, pos: MediaPosition, w: u32, h: u32) -> Put {
        Put(RefCell::new(Some(CellData::Image {
            pos: pos,
            width: w,
            height: h,
            data: data
        })))
    }
}

impl Command for Put {

    fn apply(&self, terminal: &mut Terminal) {
        if let Some(data) = self.0.borrow_mut().take() {
            terminal.write(data)
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

pub struct PutAt(RefCell<Option<CellData>>, Coords);

impl PutAt {

    pub fn new_image(data: DynamicImage, pos: MediaPosition, w: u32, h: u32, at: Coords) -> PutAt {
        PutAt(RefCell::new(Some(CellData::Image {
            pos: pos,
            width: w,
            height: h,
            data: data,
        })), at)
    }
}

impl Command for PutAt {

    fn apply(&self, terminal: &mut Terminal) {
        if let Some(data) = self.0.borrow_mut().take() {
            let coords = terminal.cursor_position();
            terminal.move_cursor(Position(self.1));
            terminal.write(data);
            terminal.move_cursor(Position(coords));
        }
    }

    fn repr(&self) -> String {
        String::from("PUT AT")
    }

}
