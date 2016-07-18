use std::ops::Index;

use datatypes::{Coords, CoordsIter, Region};

use super::section::ScreenSection;
use super::panel::Panel::*;

/// An iterator over all of the cells in a section of the screen - either one panel or the entire
/// screen.
pub struct Cells<'a, T: 'a> {
    pub(super) iter: CoordsIter,
    pub(super) section: &'a ScreenSection<T>,
}

impl<'a, T: 'a> Cells<'a, T> {
    /// The section of the screen that this iterator iterates over.
    pub fn area(&self) -> Region {
        self.iter.region()
    }
}


impl<'a, T: Index<Coords>> Iterator for Cells<'a, T> {
    type Item = &'a T::Output;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|coords| &self.section[coords])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

}

impl<'a, T: Index<Coords>> DoubleEndedIterator for Cells<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|coords| &self.section[coords])
    }
}

impl<'a, T: Index<Coords>> ExactSizeIterator for Cells<'a, T> { }

/// An iterator over all of the visible panels in the terminal's screen.
pub struct Panels<'a, T: 'a> {
    pub(super) stack: Vec<&'a ScreenSection<T>>,
}

impl<'a, T: 'a> Iterator for Panels<'a, T> {
    type Item = Cells<'a, T>;

    fn next(&mut self) -> Option<Cells<'a, T>> {
        fn cells<'a, T>(section: &'a ScreenSection<T>,
                        stack: &mut Vec<&'a ScreenSection<T>>) -> Cells<'a, T> {
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

impl<'a, T: 'a> ExactSizeIterator for Panels<'a, T> {
    fn len(&self) -> usize {
        self.stack.iter().cloned().map(ScreenSection::count_grids).sum()
    }
}
