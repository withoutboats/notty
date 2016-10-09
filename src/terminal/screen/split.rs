use std::mem;
use std::ops::Index;

use datatypes::{Coords, Region, ResizeRule, SplitKind, SaveGrid};
use datatypes::ResizeRule::*;
use datatypes::SplitKind::*;
use terminal::interfaces::Resizeable;

use super::panel::Panel;
use super::ring::Ring;
use super::section::ScreenSection;

#[derive(Debug, Eq, PartialEq)]
pub struct SplitSection<T> {
    left: Box<ScreenSection<T>>,
    right: Box<ScreenSection<T>>,
    area: Region,
    kind: SplitKind,
    rule: ResizeRule,
}

impl<T> SplitSection<T> {
    pub fn new(left: Box<ScreenSection<T>>, right: Box<ScreenSection<T>>, area: Region, kind: SplitKind) -> SplitSection<T> {
        SplitSection {
            left: left,
            right: right,
            area: area,
            kind: kind,
            rule: ResizeRule::Percentage,
        }
    }

    pub fn count_leaves(&self) -> usize {
        self.left.count_leaves() + self.right.count_leaves()
    }

    pub fn children(&self) -> (&ScreenSection<T>, &ScreenSection<T>) {
        (&self.left, &self.right)
    }

    pub fn find(&self, tag: u64) -> Option<&ScreenSection<T>> {
        let SplitSection { ref left, ref right, .. } = *self;
        left.find(tag).or_else(move || right.find(tag))
    }

    pub fn find_mut(&mut self, tag: u64) -> Option<&mut ScreenSection<T>> {
        let SplitSection { ref mut left, ref mut right, .. } = *self;
        left.find_mut(tag).or_else(move || right.find_mut(tag))
    }
}

impl<T: Resizeable> SplitSection<T> {
    pub fn shift_into(&mut self, area: Region) {
        self.area = area;
        self.resize(area.width(), area.height());
    }

    pub fn adjust_split(&mut self, new_kind: SplitKind) {
        let (new_kind, l_area, r_area) = self.area.split(new_kind, self.rule);
        self.kind = new_kind;
        self.left.resize(l_area.width(), l_area.height());
        self.right.resize(r_area.width(), r_area.height());
    }

    pub fn unsplit(mut self, save: SaveGrid) -> Ring<Panel<T>> {
        let mut saved_ring = match save {
            SaveGrid::Left  => mem::replace(&mut self.left.ring, Ring::new(Panel::Dead)),
            SaveGrid::Right => mem::replace(&mut self.right.ring, Ring::new(Panel::Dead)),
        };

        let (width, height) = (self.area.width(), self.area.height());
        for panel in &mut saved_ring {
            panel.resize(width, height);
        }
        saved_ring
    }
}

impl<T: Resizeable> Resizeable for SplitSection<T> {
    fn dims(&self) -> (u32, u32) {
        (self.area.width(), self.area.height())
    }

    fn resize_width(&mut self, width: u32) {
        let height = self.area.height();
        self.resize(width, height);
    }

    fn resize_height(&mut self, height: u32) {
        let width = self.area.width();
        self.resize(width, height);
    }

    fn resize(&mut self, width: u32, height: u32) {
        let(kind, l_area, r_area) = resize_split(self.area, self.area.resized(width, height), self.kind, self.rule);
        self.area = self.area.resized(width, height);
        self.left.shift_into(l_area);
        self.right.shift_into(r_area);
        self.kind = kind;
    }
}

fn resize_split(old_area: Region, new_area: Region, kind: SplitKind, rule: ResizeRule)
        -> (SplitKind, Region, Region) {
    let kind = match (kind, rule) {
        (Horizontal(n), Percentage) if old_area.height() != new_area.height()       =>
            Horizontal((n as f32 / old_area.height() as f32 * new_area.height() as f32) as u32),
        (Vertical(n), Percentage) if old_area.width() != new_area.width()           =>
            Vertical((n as f32 / old_area.width() as f32 * new_area.width() as f32) as u32),
        (Horizontal(n), MaxLeftTop) if new_area.height() > old_area.height()        =>
            Horizontal(new_area.height() - old_area.height() + n),
        (Vertical(n), MaxLeftTop) if new_area.width() > old_area.width()            =>
            Vertical(new_area.width() - old_area.width() + n),
        _ => kind,
    };
    new_area.split(kind, rule)
}

impl<T: Index<Coords>> Index<Coords> for SplitSection<T> {
    type Output = T::Output;
    fn index(&self, Coords { x, y }: Coords) -> &Self::Output {
        match self.kind {
            Horizontal(n) if y < n  => &self.left[Coords { x: x, y: y }],
            Vertical(n) if x < n    => &self.left[Coords { x: x, y: y }],
            Horizontal(n) if n <= y => &self.right[Coords { x: x, y: y - n }],
            Vertical(n) if n <= x   => &self.right[Coords { x: x - n, y: y }],
            _                       => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    pub use terminal::screen::tests::*;

    fn test(into: Region, rule: ResizeRule, expected_kind: SplitKind, expected_left: Region, expected_right: Region) {
        let old_a = Region::new(0, 0, 8, 8);
        let old_kind = Horizontal(4);
        let (kind, left, right) = super::resize_split(old_a, into, old_kind, rule);
        assert_eq!(kind, expected_kind);
        assert_eq!(left, expected_left);
        assert_eq!(right, expected_right);
    }

    mod into_4_4 {
        use super::*;

        fn test_4_4(rule: ResizeRule, expected_kind: SplitKind, expected_left: Region, expected_right: Region) {
            super::test(Region::new(0, 0, 4, 4), rule, expected_kind, expected_left, expected_right);
        }

        #[test]
        fn max_left() {
            test_4_4(MaxLeftTop, Horizontal(3), Region::new(0, 0, 4, 3), Region::new(0, 3, 4, 4));
        }

        #[test]
        fn max_right() {
            test_4_4(MaxRightBottom, Horizontal(1), Region::new(0, 0, 4, 1), Region::new(0, 1, 4, 4));
        }

        #[test]
        fn percent() {
            test_4_4(Percentage, Horizontal(2), Region::new(0, 0, 4, 2), Region::new(0, 2, 4, 4));
        }
    }

    mod into_6_6 {
        use super::*;

        fn test_6_6(rule: ResizeRule, expected_kind: SplitKind, expected_left: Region, expected_right: Region) {
            super::test(Region::new(0, 0, 6, 6), rule, expected_kind, expected_left, expected_right);
        }

        #[test]
        fn max_left() {
            test_6_6(MaxLeftTop, Horizontal(4), Region::new(0, 0, 6, 4), Region::new(0, 4, 6, 6));
        }

        #[test]
        fn max_right() {
            test_6_6(MaxRightBottom, Horizontal(2), Region::new(0, 0, 6, 2), Region::new(0, 2, 6, 6));
        }

        #[test]
        fn percent() {
            test_6_6(Percentage, Horizontal(3), Region::new(0, 0, 6, 3), Region::new(0, 3, 6, 6));
        }
    }

    mod into_16_16 {
        use super::*;

        fn test_16_16(rule: ResizeRule, expected_kind: SplitKind, expected_left: Region, expected_right: Region) {
            super::test(Region::new(0, 0, 16, 16), rule, expected_kind, expected_left, expected_right);
        }

        #[test]
        fn max_left() {
            test_16_16(MaxLeftTop, Horizontal(12), Region::new(0, 0, 16, 12), Region::new(0, 12, 16, 16));
        }

        #[test]
        fn max_right() {
            test_16_16(MaxRightBottom, Horizontal(4), Region::new(0, 0, 16, 4), Region::new(0, 4, 16, 16));
        }

        #[test]
        fn percent() {
            test_16_16(Percentage, Horizontal(8), Region::new(0, 0, 16, 8), Region::new(0, 8, 16, 16));
        }
    }
}
