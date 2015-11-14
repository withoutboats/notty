use std::io::Write;
use std::mem;
use std::ops::{Deref, DerefMut};

mod cell;
mod char_grid;
mod cursor;
mod grid;
mod input;
mod styles;

use datatypes::{InputMode, Key};

pub use self::cell::CharCell;
pub use self::char_grid::CharGrid;
pub use self::cursor::Cursor;
pub use self::grid::Grid;
pub use self::styles::Styles;

use self::input::Input;

pub struct Terminal {
    pub width: u32,
    pub height: u32,
    title: String,
    active: CharGrid,
    inactive: Vec<CharGrid>,
    tty: Input,
}

impl Terminal {

    pub fn new<W: Write + 'static>(width: u32, height: u32, tty: W) -> Terminal {
        let grid = CharGrid::new(width, height, false, true);
        let tty = Input::new(tty);
        Terminal {
            width: width,
            height: height,
            title: String::new(),
            active: grid,
            inactive: Vec::new(),
            tty: tty,
        }
    }

    pub fn send_input(&mut self, key: Key, press: bool) {
        self.tty.process(key, press);
    }

    pub fn push_buffer(&mut self, scroll_x: bool, scroll_y: bool) {
        let mut grid = CharGrid::new(self.width, self.height, scroll_x, scroll_y);
        mem::swap(&mut grid, &mut self.active);
        self.inactive.push(grid);
    }

    pub fn pop_buffer(&mut self) {
        self.inactive.pop().map(|grid| self.active = grid);
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn set_input_mode(&mut self, mode: InputMode) {
        self.tty.set_mode(mode);
    }

    pub fn bell(&mut self) {
        println!("BELL");
    }

    pub fn set_visible_height(&mut self, rows: u32) {
        self.active.set_height(rows);
        self.height = rows;
    }

    pub fn set_visible_width(&mut self, cols: u32) {
        self.active.set_width(cols);
        self.width = cols;
    }

}

impl Deref for Terminal {
    type Target = CharGrid;
    fn deref(&self) -> &CharGrid {
        &self.active
    }
}

impl DerefMut for Terminal {
    fn deref_mut(&mut self) -> &mut CharGrid {
        &mut self.active
    }
}
