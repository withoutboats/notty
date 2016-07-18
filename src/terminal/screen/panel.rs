use datatypes::{Region, ResizeRule, SplitKind};
use datatypes::ResizeRule::*;
use datatypes::SplitKind::*;
use terminal::interfaces::Resizeable;

use super::section::ScreenSection;
use self::Panel::*;

#[derive(Debug, Eq, PartialEq)]
pub enum Panel<T> {
    Grid(T),
    Split {
        kind: SplitKind,
        left: Box<ScreenSection<T>>,
        right: Box<ScreenSection<T>>,
    },
    DeadGrid,
}

impl<T: Resizeable> Panel<T> {
    pub fn resize(&mut self, old_area: Region, new_area: Region, rule: ResizeRule) {
        match *self {
            Grid(ref mut grid) => {
                grid.resize(new_area.width(), new_area.height());
            }
            Split { ref mut left, ref mut right, ref mut kind } => {
                let (new_kind, l_area, r_area) = resize_split(old_area, new_area, *kind, rule);
                *kind = new_kind;
                left.resize(l_area, rule);
                right.resize(r_area, rule);
            }
            DeadGrid => unreachable!()
        }
    }
}

impl<T> Panel<T> {
    pub fn is_grid(&self) -> bool {
        if let Grid(_) = *self { true } else { false }
    }

    pub fn find(&self, tag: u64) -> Option<&ScreenSection<T>> {
        if let Split { ref left, ref right, .. } = *self {
            left.find(tag).or_else(move || right.find(tag))
        } else { None }
    }

    pub fn find_mut(&mut self, tag: u64) -> Option<&mut ScreenSection<T>> {
        if let Split { ref mut left, ref mut right, .. } = *self {
            left.find_mut(tag).or_else(move || right.find_mut(tag))
        } else { None }
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

#[cfg(test)]
mod tests {
    pub use terminal::screen::tests::*;

    mod grid_panel {
        use super::*;
        
        const PANEL: Panel<MockGrid> = Panel::Grid(GRID);

        #[test]
        fn resize() {
            let mut panel = PANEL;
            panel.resize(OLD_AREA, NEW_AREA, Percentage);
            assert_eq!(panel, Panel::Grid(MockGrid(6, 10)));
        }

        #[test]
        fn is_grid() {
            assert!(PANEL.is_grid());
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

        fn split_panel() -> Panel<MockGrid> {
            Panel::Split {
                kind: SplitKind::Horizontal(4),
                left: Box::new(ScreenSection::new(0, Region::new(0, 0, 8, 4), false)),
                right: Box::new(ScreenSection::new(1, Region::new(0, 4, 8, 8), false)),
            }
        }

        #[test]
        fn is_grid() {
            assert!(!split_panel().is_grid());
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

    mod resize_split {
        pub use super::*;

        fn test(into: Region, rule: ResizeRule, expected_kind: SplitKind, expected_left: Region, expected_right: Region) {
            let old_a = Region::new(0, 0, 8, 8);
            let old_kind = Horizontal(4);
            let (kind, left, right) = super::super::resize_split(old_a, into, old_kind, rule);
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
}
