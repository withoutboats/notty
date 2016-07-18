pub use datatypes::{GridSettings, Region, ResizeRule, SplitKind, SaveGrid};
pub use datatypes::ResizeRule::*;
pub use datatypes::SplitKind::*;
pub use terminal::interfaces::{Resizeable, ConstructGrid};
pub use super::panel::Panel;
pub use super::panel::Panel::*;
pub use super::ring::Ring;
pub use super::section::ScreenSection;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct MockGrid(pub u32, pub u32);

impl Resizeable for MockGrid {
    fn resize_width(&mut self, width: u32) {
        self.0 = width;
    }

    fn resize_height(&mut self, height: u32) {
        self.1 = height;
    }
}

impl ConstructGrid for MockGrid {
    fn new(settings: GridSettings) -> MockGrid {
        MockGrid(settings.width, settings.height)
    }
}

pub const GRID: MockGrid = MockGrid(8, 8);
pub const OLD_AREA: Region = Region { left: 0, top: 2, right: 8, bottom: 10 };
pub const NEW_AREA: Region = Region { left: 1, top: 1, right: 7, bottom: 11 };
