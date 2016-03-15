use datatypes::{Region, SplitKind, ResizeRule};
use datatypes::SplitKind::*;
use datatypes::ResizeRule::*;

use terminal::CharGrid;

use super::section::ScreenSection;
use self::Panel::*;

pub enum Panel {
    Grid(CharGrid),
    Split {
        kind: SplitKind,
        left: Box<ScreenSection>,
        right: Box<ScreenSection>,
    },
    DeadGrid,
}

impl Panel {

    pub fn is_grid(&self) -> bool {
        if let Grid(_) = *self { true } else { false }
    }

    pub fn find(&self, tag: u64) -> Option<&ScreenSection> {
        if let Split { ref left, ref right, .. } = *self {
            left.find(tag).or_else(move || right.find(tag))
        } else { None }
    }

    pub fn find_mut(&mut self, tag: u64) -> Option<&mut ScreenSection> {
        if let Split { ref mut left, ref mut right, .. } = *self {
            left.find_mut(tag).or_else(move || right.find_mut(tag))
        } else { None }
    }

    pub fn resize(&mut self, old_a: Region, new_a: Region, rule: ResizeRule) {
        match *self {
            Grid(ref mut grid) => {
                grid.resize(new_a);
            }
            Split { ref mut left, ref mut right, ref mut kind } => {
                *kind = match (*kind, rule) {
                    (Horizontal(mut n), Percentage) => {
                        n = (n as f32 / old_a.height() as f32 * new_a.height() as f32) as u32;
                        Horizontal(n)
                    }
                    (Vertical(mut n), Percentage)   => {
                        n = (n as f32 / old_a.width() as f32 * new_a.width() as f32) as u32;
                        Vertical(n)
                    }
                    _                               => *kind
                };
                let (new_kind, l_area, r_area) = new_a.split(*kind, rule);
                *kind = new_kind;
                left.resize(l_area, rule);
                right.resize(r_area, rule);
            }
            DeadGrid => unreachable!()
        }
    }

}
