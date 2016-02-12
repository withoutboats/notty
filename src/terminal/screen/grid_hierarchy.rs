use std::cmp;

use datatypes::Region;
use terminal::char_grid::CharGrid;

use self::GridHierarchy::*;
use self::SplitKind::*;
use self::ResizeRule::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SplitKind {
    Horizontal(u32),
    Vertical(u32),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SaveGrid {
    Left, Right, Dont
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ResizeRule {
    Percentage,
    MaxLeftTop,
    MaxRightBottom,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum GridHierarchy {
    Grid(u64, Region),
    Split {
        tag: u64,
        area: Region,
        kind: SplitKind,
        left: Box<GridHierarchy>,
        right: Box<GridHierarchy>,
    },
    Stack {
        tag: u64,
        area: Region,
        stack: Vec<GridHierarchy>,
    }
}

impl GridHierarchy {

    pub fn find(&self, tag: u64) -> Option<&GridHierarchy> {
        match *self {
            _ if self.tag() == tag => Some(self),
            Split { ref left, ref right, .. } => left.find(tag).or_else(move || right.find(tag)),
            Stack { ref stack, .. } => stack.iter().flat_map(|grid| grid.find(tag)).next(),
            _ => None
        }
    }

    pub fn find_mut(&mut self, tag: u64) -> Option<&mut GridHierarchy> {
        match *self {
            _ if self.tag() == tag => Some(self),
            Split { ref mut left, ref mut right, .. } =>
                left.find_mut(tag).or_else(move || right.find_mut(tag)),
            Stack { ref mut stack, .. } =>
                stack.iter_mut().flat_map(|grid| grid.find_mut(tag)).next(),
            _ => None
        }
    }

    pub fn is_grid(&self) -> bool {
        match *self {
            Grid(..)    => true,
            _           => false,
        }
    }

    pub fn split<T, F1, F2>(&mut self,
                            grids: &mut T,
                            add_grid: F1,
                            resize_grid: F2,
                            save: SaveGrid,
                            kind: SplitKind,
                            rule: ResizeRule,
                            ltag: u64,
                            rtag: u64)
    where F1: Fn(&mut T, u64, CharGrid), F2: Fn(&mut T, u64, Region) {
        let (l_region, r_region) = self.split_region(kind, rule);
        match save {
            SaveGrid::Left  => {
                let mut l_grid_h = self.make_new(ltag);
                l_grid_h.resize(grids, l_region, &resize_grid, rule);
                add_grid(grids, rtag,
                         CharGrid::new(r_region.width(), r_region.height(), false, false));
                *self = GridHierarchy::Split {
                    tag: self.tag(),
                    area: self.area(),
                    kind: kind,
                    left: Box::new(l_grid_h),
                    right: Box::new(Grid(rtag, r_region)),
                }
            }
            SaveGrid::Right => {
                add_grid(grids, ltag,
                         CharGrid::new(l_region.width(), l_region.height(), false, false));
                let mut r_grid_h = self.make_new(rtag);
                r_grid_h.resize(grids, r_region, &resize_grid, rule);
                *self = GridHierarchy::Split {
                    tag: self.tag(),
                    area: self.area(),
                    kind: kind,
                    left: Box::new(Grid(ltag, l_region)),
                    right: Box::new(r_grid_h),
                }
            }
            SaveGrid::Dont  => {
                add_grid(grids, ltag,
                         CharGrid::new(l_region.width(), l_region.height(), false, false));
                add_grid(grids, rtag,
                         CharGrid::new(r_region.width(), r_region.height(), false, false));
                *self = GridHierarchy::Split {
                    tag: self.tag(),
                    area: self.area(),
                    kind: kind,
                    left: Box::new(Grid(ltag, l_region)),
                    right: Box::new(Grid(rtag, r_region)),
                }
            }
        }
    }

    pub fn remove<T, F1, F2>(&mut self,
                             grids: &mut T,
                             remove_grid: F1,
                             resize_grid: F2,
                             tag: u64,
                             rule: ResizeRule)
    where F1: Fn(&mut T, u64), F2: Fn(&mut T, u64, Region) {
        let replacement_grid = if let Some(parent_grid) = self.find_parent(tag) {
            match *parent_grid {
                Grid(..) => unreachable!(),
                Stack { ref mut stack, .. } => {
                    //TODO does this remove dead grids from the hash map?
                    if stack.last().unwrap().tag() == tag {
                        stack.pop();
                    } else {
                        let idx = stack.iter().enumerate()
                                       .filter(|&(_, grid)| grid.tag() == tag)
                                       .map(|(idx, _)| idx)
                                       .next().unwrap();
                        stack.remove(idx);
                    }
                    if stack.len() == 1 { Some(stack.last().unwrap().clone()) }
                    else { None }
                }
                Split { ref mut left, ref mut right, area, .. } => {
                    if left.tag() == tag {
                        for grid in left.grid_tags() { remove_grid(grids, grid) }
                        right.resize(grids, area, &resize_grid, rule);
                        Some((**right).clone())
                    } else if right.tag() == tag {
                        for grid in right.grid_tags() { remove_grid(grids, grid) };
                        left.resize(grids, area, &resize_grid, rule);
                        Some((**left).clone())
                    } else { unreachable!() }
                }
            }
        } else { None };
        if let Some(grid) = replacement_grid {
            *self = grid;
        }
    }

    pub fn resize<T>(&mut self,
                     grids: &mut T,
                     new_a: Region,
                     resize_grid: &Fn(&mut T, u64, Region),
                     rule: ResizeRule)
    {
        match *self {
            Grid(tag, ref mut area) => {
                *area = new_a;
                resize_grid(grids, tag, new_a);
            }
            Stack { ref mut area, ref mut stack, .. } => {
                *area = new_a;
                for grid in stack {
                    grid.resize(grids, new_a, resize_grid, rule);
                }
            }
            Split { ref mut left, ref mut right, ref mut area, kind, .. } => {
                let kind = match (kind, rule) {
                    (Horizontal(mut n), Percentage) => {
                        n = (n as f32 / area.height() as f32 * new_a.height() as f32) as u32;
                        Horizontal(n)
                    }
                    (Vertical(mut n), Percentage)   => {
                        n = (n as f32 / area.width() as f32 * new_a.width() as f32) as u32;
                        Vertical(n)
                    }
                    _                               => kind
                };
                *area = new_a;
                let (l_area, r_area) = split_region(new_a, kind, rule);
                left.resize(grids, l_area, resize_grid, rule);
                right.resize(grids, r_area, resize_grid, rule);
            }
        }
    }

    fn make_new(&self, tag: u64) -> GridHierarchy {
        let mut new = self.clone();
        new.set_tag(tag);
        new
    }

    fn set_tag(&mut self, new_tag: u64) {
        match *self {
            Grid(ref mut tag, _) | Split { ref mut tag, .. } | Stack { ref mut tag, .. }
                => *tag = new_tag
        }
    }

    fn split_region(&self, kind: SplitKind, rule: ResizeRule) -> (Region, Region) {
        split_region(self.area(), kind, rule)
    }

    pub fn area(&self) -> Region {
        match *self {
            Grid(_, area) | Split { area, .. } | Stack { area, .. } => area
        }
    }

    fn tag(&self) -> u64 {
        match *self {
            Grid(tag, _) | Split { tag, .. } | Stack { tag, .. } => tag
        }
    }

    fn grid_tags(&self) -> Vec<u64> {
        fn _grid_tags(grid: &GridHierarchy, tags: &mut Vec<u64>) {
            match *grid {
                Grid(tag, _) => tags.push(tag),
                Split { ref left, ref right, .. } => {
                    _grid_tags(left, tags);
                    _grid_tags(right, tags);
                }
                Stack { ref stack, .. } => {
                    for grid in stack { _grid_tags(grid, tags); }
                }
            }
        }
        let mut v = vec![];
        _grid_tags(self, &mut v);
        v
    }

    // NOTE: This performs two dives (_find_parent and find_mut) because of lexical lifetimes
    fn find_parent(&mut self, tag: u64) -> Option<&mut GridHierarchy> {
        fn _find_parent(grid: &GridHierarchy, tag: u64) -> Option<u64> {
            match *grid {
                Grid(..) => None,
                Stack { ref stack, .. } => {
                    stack.iter().flat_map(|child| {
                        if child.tag() == tag { Some(grid.tag()) }
                        else { _find_parent(child, tag) }
                    }).next()
                }
                Split { ref left, ref right, .. } => {
                    if left.tag() == tag || right.tag() == tag { Some(grid.tag()) }
                    else { _find_parent(left, tag).or_else(|| _find_parent(right, tag)) }
                }
            }
        }
        _find_parent(self, tag).and_then(move |tag| self.find_mut(tag))
    }
    
}

fn split_region(region: Region, kind: SplitKind, rule: ResizeRule) -> (Region, Region) {
    match (kind, rule) {
        (Horizontal(n), MaxLeftTop) | (Horizontal(n), Percentage)   => (
            Region { bottom: cmp::min(region.top + n, region.bottom - 1), ..region },
            Region { top: cmp::min(region.top + n, region.bottom - 1), ..region }
        ),
        (Horizontal(n), MaxRightBottom)                             => (
            Region { bottom: cmp::max(region.bottom.saturating_sub(n), region.top + 1), ..region },
            Region { top: cmp::max(region.bottom.saturating_sub(n), region.top + 1), ..region },
        ),
        (Vertical(n), MaxLeftTop) | (Vertical(n), Percentage)       => (
            Region { right: cmp::min(region.left + n, region.right - 1), ..region },
            Region { left: cmp::min(region.left + n, region.right - 1), ..region }
        ),
        (Vertical(n), MaxRightBottom)                               => (
            Region { right: cmp::max(region.right.saturating_sub(n), region.left + 1), ..region },
            Region { left: cmp::max(region.right.saturating_sub(n), region.left + 1), ..region },
        ),
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use super::GridHierarchy::*;

    use datatypes::Region;

    // The hierarchy this sets up is:
    //  0
    //  | \
    //  1  2
    //  | \
    //  3 0x0beefdad
    // Beef Dad is the needle for these tests.
    fn setup_grid_hierarchy() -> GridHierarchy {
        Split {
            tag: 0,
            area: Region::new(0, 0, 8, 8),
            kind: SplitKind::Vertical(4),
            left: Box::new(Split {
                tag: 1,
                area: Region::new(0, 0, 4, 8),
                kind: SplitKind::Horizontal(4),
                left: Box::new(Grid(3), Region::new(0, 0, 4, 4)),
                right: Box::new(Grid(0x0beefdad), Region::new(0, 4, 4, 8)),
            }),
            right: Box::new(Grid(2), Region::new(4, 0, 8, 8)),
        }
    }

    // After this test:
    // 0
    // | \
    // 3  2
    #[test]
    fn remove_a_tag() {
        let mut gh = setup_grid_hierarchy();
        gh.remove(0x0beefdad);
        assert_eq!(gh, Split {
            tag: 0,
            area: Region::new(0, 0, 8, 8)
            kind: SplitKind::Vertical(4),
            left: Box::new(Grid(3, Region::new(0, 0, 4, 8))),
            right: Box::new(Grid(2), Region::new(4, 0, 8, 8)),
        })
    }

    // After this test:
    // 0
    // | \
    // 1  2
    // | \
    // 3 0x0badcafe
    #[test]
    fn replace_a_tag() {
        let mut gh = setup_grid_hierarchy();
        gh.replace(0x0beefdad, |_| GridHierarchy::Grid(0x0badcafe));
        assert_eq!(gh, Split {
            tag: 0,
            kind: SplitKind::Horizontal(2),
            left: Box::new(Split {
                tag: 1,
                kind: SplitKind::Horizontal(2),
                left: Box::new(Grid(3)),
                right: Box::new(Grid(0x0badcafe)),
            }),
            right: Box::new(Grid(2)),
        })
    }
}
