use std::ops::{Deref, DerefMut};

use datatypes::{CoordsIter, Region};
use terminal::char_grid::{CharGrid, CharCell};

mod stack;
mod panel;

use self::panel::Panel;

pub use self::panel::{ResizeRule, SplitKind, SaveGrid};
pub use self::stack::Stack;

pub struct Screen {
    active: u64,
    screen: Panel,
}

impl Screen {

    pub fn new(width: u32, height: u32) -> Screen {
        Screen {
            active: 0,
            screen: Panel::new(0, Region::new(0, 0, width, height)),
        }
    }

    pub fn resize(&mut self, width: Option<u32>, height: Option<u32>, rule: ResizeRule) {
        let new_a = match (width, height) {
            (Some(w), Some(h))  => Region::new(0, 0, w, h),
            (Some(w), None)     => Region::new(0, 0, w, self.screen.area.bottom),
            (None,    Some(h))  => Region::new(0, 0, self.screen.area.right, h),
            (None,    None)     => return
        };
        self.screen.resize(new_a, rule);
    }

    pub fn switch(&mut self, tag: u64) {
        if self.find(Some(tag)).map_or(false, Panel::is_grid) {
            self.active = tag;
        }
    }

    pub fn split(&mut self, save: SaveGrid, kind: SplitKind, rule: ResizeRule,
                 split_tag: Option<u64>, l_tag: u64, r_tag: u64) {
        self.find_mut(split_tag).map(|panel| panel.split(save, kind, rule, l_tag, r_tag));
    }

    pub fn unsplit(&mut self, save: SaveGrid, rule: ResizeRule, unsplit_tag: Option<u64>) {
        self.find_mut(unsplit_tag).map(|panel| panel.unsplit(save, rule));
    }

    pub fn push(&mut self, tag: Option<u64>) {
        self.find_mut(tag).map(Panel::push);
    }

    pub fn pop(&mut self, tag: Option<u64>) {
        self.find_mut(tag).map(Panel::pop);
    }

    pub fn cells(&self) -> Cells {
        Cells {
            iter: CoordsIter::from_region(self.screen.area),
            screen: &self.screen
        }
    }

    fn find(&self, tag: Option<u64>) -> Option<&Panel> {
        self.screen.find(tag.unwrap_or(self.active))
    }

    fn find_mut(&mut self, tag: Option<u64>) -> Option<&mut Panel> {
        self.screen.find_mut(tag.unwrap_or(self.active))
    }

}

impl Deref for Screen {
    type Target = CharGrid;
    fn deref(&self) -> &CharGrid {
        self.find(None).expect("active panel must exist").grid()
    }
}

impl DerefMut for Screen {
    fn deref_mut(&mut self) -> &mut CharGrid {
        self.find_mut(None).expect("active panel must exist").grid_mut()
    }
}

pub struct Cells<'a> {
    iter: CoordsIter,
    screen: &'a Panel,
}

impl<'a> Iterator for Cells<'a> {
    type Item = &'a CharCell;
    fn next(&mut self) -> Option<&'a CharCell> {
        self.iter.next().map(|coords| &self.screen[coords])
    }
}
