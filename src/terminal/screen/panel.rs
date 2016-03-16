use datatypes::{Region, SplitKind, ResizeRule};
use datatypes::SplitKind::*;
use datatypes::ResizeRule::*;

use terminal::CharGrid;

use super::GridFill;
use super::section::ScreenSection;
use self::Panel::*;

#[derive(Debug, Eq, PartialEq)]
pub enum Panel<T=CharGrid> where T: GridFill {
    Grid(T),
    Split {
        kind: SplitKind,
        left: Box<ScreenSection<T>>,
        right: Box<ScreenSection<T>>,
    },
    DeadGrid,
}

impl<T: GridFill> Panel<T> {

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

    pub fn resize(&mut self, old_area: Region, new_area: Region, rule: ResizeRule) {
        match *self {
            Grid(ref mut grid) => {
                grid.resize(new_area);
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

fn resize_split(old_area: Region, new_area: Region, kind: SplitKind, rule: ResizeRule)
        -> (SplitKind, Region, Region) {
    let kind = match (kind, rule) {
        (Horizontal(n), Percentage) if old_area.height() != new_area.height()       =>
            Horizontal((n as f32 / old_area.height() as f32 * new_area.height() as f32) as u32),
        (Vertical(n), Percentage) if old_area.width() != old_area.width()           =>
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

    use std::fmt::Debug;

    use datatypes::{Region, SplitKind, ResizeRule};
    use datatypes::ResizeRule::*;
    use super::super::GridFill;
    use super::super::section::ScreenSection;
    use super::*;
    use super::Panel::*;

    fn grid_panel() -> Panel<Region> {
        Grid(Region::new(0, 0, 8, 8))
    }

    fn split_panel() -> Panel<Region> {
        Split {
            kind: SplitKind::Horizontal(4),
            left: Box::new(ScreenSection::new(1, Region::new(0, 0, 8, 4))),
            right: Box::new(ScreenSection::new(2, Region::new(0, 4, 8, 8))),
        }
    }

    fn run_test<F, T>(f: F, res: [T; 2]) where F: Fn(Panel<Region>) -> T, T: PartialEq + Debug {
        assert_eq!(f(grid_panel()), res[0]);
        assert_eq!(f(split_panel()), res[1]);
    }

    fn run_resize_test(old_a: Region, new_a: Region, rule: ResizeRule,
                       res: (Region, Region, SplitKind)) {
        run_test(|mut panel| {
            panel.resize(old_a, new_a, rule);
            match panel {
                Grid(region) => Err(region),
                Split { left, right, kind } => Ok((left.area(), right.area(), kind)),
                DeadGrid => unreachable!(),
            }
        }, [Err(new_a), Ok(res)])
    }

    #[test]
    fn is_grid() {
        run_test(|panel| panel.is_grid(), [true, false]);
    }

    #[test]
    fn find() {
        run_test(|panel| panel.find(2).is_some(), [false, true]);
    }

    #[test]
    fn find_mut() {
        run_test(|mut panel| panel.find_mut(2).is_some(), [false, true]);
    }

    #[test]
    fn resize_down_max_left() {
        run_resize_test(Region::new(0, 0, 8, 8), Region::new(0, 0, 4, 4), MaxLeftTop,
            (Region::new(0, 0, 4, 3), Region::new(0, 3, 4, 4), SplitKind::Horizontal(3)));
        run_resize_test(Region::new(0, 0, 8, 8), Region::new(0, 0, 6, 6), MaxLeftTop,
            (Region::new(0, 0, 6, 4), Region::new(0, 4, 6, 6), SplitKind::Horizontal(4)));
    }

    #[test]
    fn resize_down_max_right() {
        run_resize_test(Region::new(0, 0, 8, 8), Region::new(0, 0, 4, 4), MaxRightBottom,
            (Region::new(0, 0, 4, 1), Region::new(0, 1, 4, 4), SplitKind::Horizontal(1)));
        run_resize_test(Region::new(0, 0, 8, 8), Region::new(0, 0, 6, 6), MaxRightBottom,
            (Region::new(0, 0, 6, 2), Region::new(0, 2, 6, 6), SplitKind::Horizontal(2)));
    }

    #[test]
    fn resize_down_percent() {
        run_resize_test(Region::new(0, 0, 8, 8), Region::new(0, 0, 4, 4), Percentage,
            (Region::new(0, 0, 4, 2), Region::new(0, 2, 4, 4), SplitKind::Horizontal(2)));
        run_resize_test(Region::new(0, 0, 8, 8), Region::new(0, 0, 6, 6), Percentage,
            (Region::new(0, 0, 6, 3), Region::new(0, 3, 6, 6), SplitKind::Horizontal(3)));
    }

    #[test]
    fn resize_up_max_left() {
        run_resize_test(Region::new(0, 0, 8, 8), Region::new(0, 0, 16, 16), MaxLeftTop,
            (Region::new(0, 0, 16, 12), Region::new(0, 12, 16, 16), SplitKind::Horizontal(12)));
    }

    #[test]
    fn resize_up_max_right() {
        run_resize_test(Region::new(0, 0, 8, 8), Region::new(0, 0, 16, 16), MaxRightBottom,
            (Region::new(0, 0, 16, 4), Region::new(0, 4, 16, 16), SplitKind::Horizontal(4)));
    }

    #[test]
    fn resize_up_percent() {
        run_resize_test(Region::new(0, 0, 8, 8), Region::new(0, 0, 16, 16), Percentage,
            (Region::new(0, 0, 16, 8), Region::new(0, 8, 16, 16), SplitKind::Horizontal(8)));
    }

}
