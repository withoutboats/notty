use std::ops::{Deref, DerefMut};

use datatypes::{Region, SaveGrid, SplitKind, ResizeRule};
use terminal::{CharGrid};
use terminal::interfaces::{Resizeable, ConstructGrid};

mod iter;
mod panel;
mod section;
mod split;
mod ring;
#[cfg(test)]
mod tests;

pub use self::iter::{Cells, Panels};

use self::section::ScreenSection;

const E_ACTIVE: &'static str = "Active screen section must exist.";

pub struct Screen<T=CharGrid> {
    active: u64,
    screen: ScreenSection<T>,
}

impl<T: ConstructGrid + Resizeable> Screen<T> {
    pub fn split(&mut self, save: SaveGrid, kind: SplitKind, rule: ResizeRule,
                 split_tag: Option<u64>, l_tag: u64, r_tag: u64, retain_offscreen_state: bool) {
        self.find_mut(split_tag).map(|section| section.split(save, kind, rule, l_tag, r_tag,
                                                             retain_offscreen_state));
        if split_tag.map_or(true, |tag| tag == self.active) {
            self.active = match save {
                SaveGrid::Left  => l_tag,
                SaveGrid::Right => r_tag,
            };
        }
    }
}

impl<T: ConstructGrid> Screen<T> {
    pub fn new(width: u32, height: u32) -> Screen<T> {
        Screen {
            active: 0,
            screen: ScreenSection::new(0, Region::new(0, 0, width, height), true),
        }
    }

    pub fn push(&mut self, tag: Option<u64>, retain_offscreen_state: bool) {
        self.find_mut(tag).map(|section| section.push(retain_offscreen_state));
    }
}

impl<T: Resizeable> Screen<T> {
    pub fn adjust_split(&mut self, tag: u64, kind: SplitKind) {
        self.find_mut(Some(tag)).map(|section| section.adjust_split(kind));
    }

    pub fn unsplit(&mut self, save: SaveGrid, tag: u64) {
        if let Some((left, right)) = self.screen.find(tag).and_then(ScreenSection::children) {
            if self.active == left.tag() || self.active == right.tag() {
                self.active = tag;
            }
        }
        self.find_mut(Some(tag)).map(|section| section.unsplit(save));
    }
}

impl<T: Resizeable> Resizeable for Screen<T> {
    fn dims(&self) -> (u32, u32) {
        self.screen.dims()
    }

    fn resize_width(&mut self, width: u32) {
        self.screen.resize_width(width);
    }

    fn resize_height(&mut self, height: u32) {
        self.screen.resize_height(height);
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.screen.resize(width, height);
    }
}

impl<T> Screen<T> {
    pub fn area(&self) -> Region {
        self.screen.area()
    }

    pub fn switch(&mut self, tag: u64) {
        if self.find(Some(tag)).map_or(false, ScreenSection::is_fill) {
            self.active = tag;
        }
    }

    pub fn pop(&mut self, tag: Option<u64>) {
        self.find_mut(tag).map(ScreenSection::pop);
    }

    pub fn rotate_down(&mut self, tag: Option<u64>) {
        self.find_mut(tag).map(ScreenSection::rotate_down);
    }

    pub fn rotate_up(&mut self, tag: Option<u64>) {
        self.find_mut(tag).map(ScreenSection::rotate_up);
    }

    pub fn cells(&self) -> Cells<T> {
        self.screen.cells()
    }

    pub fn panels(&self) -> Panels<T> {
        self.screen.panels()
    }

    fn find(&self, tag: Option<u64>) -> Option<&ScreenSection<T>> {
        self.screen.find(tag.unwrap_or(self.active))
    }

    fn find_mut(&mut self, tag: Option<u64>) -> Option<&mut ScreenSection<T>> {
        self.screen.find_mut(tag.unwrap_or(self.active))
    }

}

impl Deref for Screen<CharGrid> {
    type Target = CharGrid;
    fn deref(&self) -> &CharGrid {
        self.find(None).expect(E_ACTIVE).fill()
    }
}

impl DerefMut for Screen<CharGrid> {
    fn deref_mut(&mut self) -> &mut CharGrid {
        self.find_mut(None).expect(E_ACTIVE).fill_mut()
    }
}
