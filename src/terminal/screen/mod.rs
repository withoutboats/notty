use std::ops::{Deref, DerefMut};

use datatypes::{Region, SaveGrid, SplitKind, ResizeRule, CoordsIter};
use terminal::char_grid::{CharGrid, CharCell};

mod panel;
mod section;
mod ring;

use self::section::ScreenSection;
use self::panel::Panel::*;

pub trait GridFill {
    fn new(u32, u32, bool) -> Self;
    fn resize(&mut self, Region);
}

impl GridFill for CharGrid {
    fn new(width: u32, height: u32, expand: bool) -> CharGrid {
        CharGrid::new(width, height, expand)
    }
    fn resize(&mut self, area: Region) { self.resize_window(area); }
}

impl GridFill for Region {
    fn new(width: u32, height: u32, _: bool) -> Region {
        Region::new(0, 0, width, height)
    }
    fn resize(&mut self, area: Region) { *self = Region::new(0, 0, area.width(), area.height()) }
}

pub struct Screen {
    active: u64,
    screen: ScreenSection,
}

impl Screen {

    pub fn new(width: u32, height: u32) -> Screen {
        Screen {
            active: 0,
            screen: ScreenSection::new(0, Region::new(0, 0, width, height), true),
        }
    }

    pub fn area(&self) -> Region {
        self.screen.area()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.screen.resize(Region::new(0, 0, width, height), ResizeRule::Percentage);
    }

    pub fn switch(&mut self, tag: u64) {
        if self.find(Some(tag)).map_or(false, ScreenSection::is_grid) {
            self.active = tag;
        }
    }

    pub fn split(&mut self, save: SaveGrid, kind: SplitKind, rule: ResizeRule,
                 split_tag: Option<u64>, l_tag: u64, r_tag: u64, retain_offscreen_state: bool) {
        self.find_mut(split_tag).map(|section| section.split(save, kind, rule, l_tag, r_tag,
                                                             retain_offscreen_state));
    }

    pub fn unsplit(&mut self, save: SaveGrid, unsplit_tag: Option<u64>) {
        self.find_mut(unsplit_tag).map(|section| section.unsplit(save));
    }

    pub fn adjust_split(&mut self, adjust_tag: Option<u64>, kind: SplitKind, rule: ResizeRule) {
        self.find_mut(adjust_tag).map(|section| section.adjust_split(kind, rule));
    }

    pub fn push(&mut self, tag: Option<u64>, retain_offscreen_state: bool) {
        self.find_mut(tag).map(|section| section.push(retain_offscreen_state));
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

    pub fn cells(&self) -> Cells {
        self.screen.cells()
    }

    pub fn panels(&self) -> Panels {
        self.screen.panels()
    }

    fn find(&self, tag: Option<u64>) -> Option<&ScreenSection> {
        self.screen.find(tag.unwrap_or(self.active))
    }

    fn find_mut(&mut self, tag: Option<u64>) -> Option<&mut ScreenSection> {
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

/// An iterator over all of the cells in a section of the screen - either one panel or the entire
/// screen.
pub struct Cells<'a> {
    iter: CoordsIter,
    screen: &'a ScreenSection,
}

impl<'a> Cells<'a> {
    /// The section of the screen that this iterator iterates over.
    pub fn area(&self) -> Region {
        self.iter.region()
    }
}

impl<'a> Iterator for Cells<'a> {
    type Item = &'a CharCell;

    fn next(&mut self) -> Option<&'a CharCell> {
        self.iter.next().map(|coords| &self.screen[coords])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

}

impl<'a> DoubleEndedIterator for Cells<'a> {
    fn next_back(&mut self) -> Option<&'a CharCell> {
        self.iter.next_back().map(|coords| &self.screen[coords])
    }
}

impl<'a> ExactSizeIterator for Cells<'a> { }

/// An iterator over all of the visible panels in the terminal's screen.
pub struct Panels<'a> {
    stack: Vec<&'a ScreenSection>,
}

impl<'a> Iterator for Panels<'a> {
    type Item = Cells<'a>;

    fn next(&mut self) -> Option<Cells<'a>> {
        fn cells<'a>(section: &'a ScreenSection, stack: &mut Vec<&'a ScreenSection>) -> Cells<'a> {
            match *section.top() {
                Grid(_) => Cells {
                    iter: CoordsIter::from_region(section.area()),
                    screen: section
                },
                Split { ref left, ref right, .. } => {
                    stack.push(right);
                    cells(left, stack)
                }
                _ => unreachable!()
            }
        }
        self.stack.pop().map(|section| cells(section, &mut self.stack))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }

}

impl<'a> ExactSizeIterator for Panels<'a> {
    fn len(&self) -> usize {
        self.stack.iter().cloned().map(ScreenSection::count_grids).sum()
    }
}
