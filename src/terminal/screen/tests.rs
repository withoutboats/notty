pub use datatypes::{GridSettings, Region, ResizeRule, SplitKind, SaveGrid};
pub use datatypes::ResizeRule::*;
pub use datatypes::SplitKind::*;
pub use terminal::interfaces::{Resizeable, ConstructGrid};
pub use super::panel::Panel;
pub use super::panel::Panel::*;
pub use super::ring::Ring;
pub use super::section::ScreenSection;
pub use super::split::SplitSection;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct MockFill(pub u32, pub u32);

impl Resizeable for MockFill {
    fn dims(&self) -> (u32, u32) {
        (self.0, self.1)
    }

    fn resize_width(&mut self, width: u32) {
        self.0 = width;
    }

    fn resize_height(&mut self, height: u32) {
        self.1 = height;
    }
}

impl ConstructGrid for MockFill {
    fn new(settings: GridSettings) -> MockFill {
        MockFill(settings.width, settings.height)
    }
}

pub const GRID: MockFill = MockFill(8, 8);
pub const NEW_AREA: Region = Region { left: 1, top: 1, right: 7, bottom: 11 };
