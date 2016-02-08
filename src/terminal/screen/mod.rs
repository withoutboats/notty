use std::collections::HashMap;
use std::ops::{Deref, DerefMut, Index};

use datatypes::{Coords, CoordsIter, Region};
use terminal::char_grid::{CharGrid, CharCell};

mod grid_hierarchy;

#[cfg(test)]
mod tests;

use self::grid_hierarchy::GridHierarchy;
use self::grid_hierarchy::GridHierarchy::*;

pub use self::grid_hierarchy::{SplitKind, ResizeRule, SaveGrid};

pub struct Screen {
    active_grid: u64,
    grid_hierarchy: GridHierarchy,
    grids: HashMap<u64, CharGrid>,
}

impl Screen {

    pub fn new(width: u32, height: u32) -> Screen {
        let mut grids = HashMap::new();
        grids.insert(0, CharGrid::new(width, height, false, false));
        Screen {
            active_grid: 0,
            grid_hierarchy: Grid(0, Region::new(0, 0, width, height)),
            grids: grids,
        }
    }

    pub fn resize(&mut self, width: Option<u32>, height: Option<u32>, rule: ResizeRule) {
        let Screen { ref mut grid_hierarchy, ref mut grids, .. } = *self;
        let new_a = match (width, height) {
            (Some(w), Some(h))  => Region::new(0, 0, w, h),
            (Some(w), None)     => grid_hierarchy.area().set_width(w),
            (None,    Some(h))  => grid_hierarchy.area().set_height(h),
            (None,    None)     => return
        };
        grid_hierarchy.resize(new_a, grids, rule);
    }

    pub fn split(&mut self, save: SaveGrid, kind: SplitKind, rule: ResizeRule,
                 stag: Option<u64>, ltag: u64, rtag: u64) {
        let Screen { active_grid, ref mut grid_hierarchy, ref mut grids } = *self;
        if let Some(grid) = grid_hierarchy.find_mut(stag.unwrap_or(active_grid)) {
            grid.split(grids, save, kind, rule, ltag, rtag);
        }
        if stag.unwrap_or(active_grid) == active_grid {
            self.active_grid = match save {
                SaveGrid::Left  => ltag,
                SaveGrid::Right => rtag,
                SaveGrid::Dont  => ltag,
            };
        }
    }

    pub fn remove(&mut self, tag: u64, rule: ResizeRule) {
        if tag != 0 && tag != self.active_grid {
            let Screen { ref mut grid_hierarchy, ref mut grids, .. } = *self;
            grid_hierarchy.remove(grids, tag, rule);
        }
    }

    pub fn switch(&mut self, tag: u64) {
        if self.grid_hierarchy.find(tag).map(|grid| grid.is_grid()).unwrap_or(false) {
            self.active_grid = tag;
        }
    }

}

impl Deref for Screen {
    type Target = CharGrid;
    fn deref(&self) -> &CharGrid {
        self.grids.get(&self.active_grid).unwrap()
    }
}

impl DerefMut for Screen {
    fn deref_mut(&mut self) -> &mut CharGrid {
        self.grids.get_mut(&self.active_grid).unwrap()
    }
}

impl Index<Coords> for Screen {
    type Output = CharCell;
    fn index(&self, idx: Coords) -> &CharCell {
        fn _index(grid_h: &GridHierarchy, idx: Coords) -> (u64, Coords) {
            match *grid_h {
                Grid(tag, area) => (tag, area.offset(idx)),
                Split { ref left, .. } if left.area().contains(idx) => _index(left, idx),
                Split { ref right, .. } if right.area().contains(idx) => _index(right, idx),
                Stack { ref stack, .. } => _index(stack.last().unwrap(), idx),
                _ => unreachable!()
            }
        }
        let (tag, idx) = _index(&self.grid_hierarchy, idx);
        self.grids.get(&tag).unwrap().window_idx(idx)
    }
}

impl<'a> IntoIterator for &'a Screen {
    type Item = &'a CharCell;
    type IntoIter = ScreenIter<'a>;
    fn into_iter(self) -> ScreenIter<'a> {
        ScreenIter {
            screen: self,
            iter: CoordsIter::from_region(self.grid_hierarchy.area()),
        }
    }
}

pub struct ScreenIter<'a> {
    screen: &'a Screen,
    iter: CoordsIter, 
}

impl<'a> Iterator for ScreenIter<'a> {
    type Item = &'a CharCell;
    fn next(&mut self) -> Option<&'a CharCell> {
        self.iter.next().map(|coords| &self.screen[coords])
    }
}

