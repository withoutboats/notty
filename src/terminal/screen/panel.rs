use std::mem;

use datatypes::Region;
use terminal::interfaces::Resizeable;

use super::section::ScreenSection;
use super::split::SplitSection;
use self::Panel::*;

#[derive(Debug, Eq, PartialEq)]
pub enum Panel<T> {
    Fill(T),
    Split(SplitSection<T>),
    Dead,
}

impl<T: Resizeable> Resizeable for Panel<T> {
    fn dims(&self) -> (u32, u32) {
        match *self {
            Fill(ref fill)      => fill.dims(),
            Split(ref split)    => split.dims(),
            _                   => unreachable!(),
        }
    }

    fn resize_width(&mut self, width: u32) {
        match *self {
            Fill(ref mut fill)      => fill.resize_width(width),
            Split(ref mut section)  => section.resize_width(width),
            Dead                    => unreachable!(),
        }
    }

    fn resize_height(&mut self, height: u32) {
        match *self {
            Fill(ref mut fill)      => fill.resize_height(height),
            Split(ref mut section)  => section.resize_height(height),
            Dead                    => unreachable!(),
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        match *self {
            Fill(ref mut fill)      => fill.resize(width, height),
            Split(ref mut section)  => section.resize(width, height),
            Dead                    => unreachable!(),
        }
    }
}

impl<T: Resizeable> Panel<T> {
    pub fn shift_into(&mut self, area: Region) {
        match *self {
            Fill(ref mut fill)      => fill.resize(area.width(), area.height()),
            Split(ref mut split)    => split.shift_into(area),
            _                       => unreachable!()
        }
    }
}

impl<T> Panel<T> {
    pub fn pull_split(&mut self) -> Option<SplitSection<T>> {
        match mem::replace(self, Dead) {
            Split(split)    => Some(split),
            otherwise       => {
                mem::replace(self, otherwise);
                None
            }
        }
    }

    pub fn is_fill(&self) -> bool {
        if let Fill(_) = *self { true } else { false }
    }

    pub fn find(&self, tag: u64) -> Option<&ScreenSection<T>> {
        if let Split(ref split) = *self {
            split.find(tag)
        } else { None }
    }

    pub fn find_mut(&mut self, tag: u64) -> Option<&mut ScreenSection<T>> {
        if let Split(ref mut split) = *self {
            split.find_mut(tag)
        } else { None }
    }
}

#[cfg(test)]
mod tests {
    pub use terminal::screen::tests::*;

    mod grid_panel {
        use super::*;
        
        const PANEL: Panel<MockFill> = Panel::Fill(GRID);

        #[test]
        fn resize() {
            let mut panel = PANEL;
            panel.resize(NEW_AREA.width(), NEW_AREA.height());
            assert_eq!(panel, Panel::Fill(MockFill(6, 10)));
        }

        #[test]
        fn is_fill() {
            assert!(PANEL.is_fill());
        }

        #[test]
        fn find() {
            assert!(PANEL.find(0).is_none());
        }

        #[test]
        fn find_mut() {
            let mut panel = PANEL;
            assert!(panel.find_mut(0).is_none());
        }
    }

    mod split_panel {
        use super::*;

        fn split_panel() -> Panel<MockFill> {
            Panel::Split(SplitSection::new(
                Box::new(ScreenSection::new(0, Region::new(0, 0, 8, 4), false)),
                Box::new(ScreenSection::new(1, Region::new(0, 4, 8, 8), false)),
                Region::new(0, 0, 8, 8),
                SplitKind::Horizontal(4),
            ))
        }

        #[test]
        fn is_fill() {
            assert!(!split_panel().is_fill());
        }

        #[test]
        fn find() {
            assert_eq!(split_panel().find(0), Some(&ScreenSection::new(0, Region::new(0, 0, 8, 4), false)));
        }

        #[test]
        fn find_mut() {
            assert_eq!(split_panel().find_mut(1), Some(&mut ScreenSection::new(1, Region::new(0, 4, 8, 8), false)));
        }
    }
}
