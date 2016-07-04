use datatypes::{CoordsIter, Region};
use terminal::char_grid::CharCell;

use super::section::ScreenSection;
use super::panel::Panel::*;

/// An iterator over all of the cells in a section of the screen - either one panel or the entire
/// screen.
pub struct Cells<'a> {
    pub(super) iter: CoordsIter,
    pub(super) section: &'a ScreenSection,
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
        self.iter.next().map(|coords| &self.section[coords])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

}

impl<'a> DoubleEndedIterator for Cells<'a> {
    fn next_back(&mut self) -> Option<&'a CharCell> {
        self.iter.next_back().map(|coords| &self.section[coords])
    }
}

impl<'a> ExactSizeIterator for Cells<'a> { }

/// An iterator over all of the visible panels in the terminal's screen.
pub struct Panels<'a> {
    pub(super) stack: Vec<&'a ScreenSection>,
}

impl<'a> Iterator for Panels<'a> {
    type Item = Cells<'a>;

    fn next(&mut self) -> Option<Cells<'a>> {
        fn cells<'a>(section: &'a ScreenSection, stack: &mut Vec<&'a ScreenSection>) -> Cells<'a> {
            match *section.top() {
                Grid(_) => Cells {
                    iter: CoordsIter::from_region(section.area()),
                    section: section
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
