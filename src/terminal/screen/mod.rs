use std::ops::{Deref, DerefMut, Index};

use datatypes::{Coords, CoordsIter, Region};
use terminal::char_grid::{CharGrid, CharCell};

mod grid_hierarchy;
mod grid_map;

//#[cfg(test)]
//mod tests;

use self::grid_hierarchy::GridHierarchy;
use self::grid_hierarchy::GridHierarchy::*;
use self::grid_map::GridMap;

pub use self::grid_hierarchy::{SplitKind, ResizeRule, SaveGrid};

pub struct Screen {
    grids: GridMap,
    grid_hierarchy: GridHierarchy,
}

impl Screen {

    pub fn new(width: u32, height: u32) -> Screen {
        Screen {
            grids: GridMap::new(width, height),
            grid_hierarchy: Grid(0, Region::new(0, 0, width, height)),
        }
    }

    pub fn resize(&mut self, width: Option<u32>, height: Option<u32>, rule: ResizeRule) {
        let Screen { ref mut grid_hierarchy, ref mut grids, .. } = *self;
        let new_a = match (width, height) {
            (Some(w), Some(h))  => Region::new(0, 0, w, h),
            (Some(w), None)     => Region::new(0, 0, w, grid_hierarchy.area().bottom),
            (None,    Some(h))  => Region::new(0, 0, grid_hierarchy.area().right, h),
            (None,    None)     => return
        };
        grid_hierarchy.resize(grids,
                              new_a,
                              &|grids, tag, area| grids.resize(tag, area),
                              rule);
    }

    pub fn split(&mut self, save: SaveGrid, kind: SplitKind, rule: ResizeRule,
                 stag: Option<u64>, ltag: u64, rtag: u64) {
        let Screen { ref mut grid_hierarchy, ref mut grids } = *self;
        if let Some(grid) = grid_hierarchy.find_mut(stag.unwrap_or(grids.active_tag())) {
            grid.split(grids, GridMap::insert, GridMap::resize, save, kind, rule, ltag, rtag);
        }
        if stag.map_or(true, |stag| grids.is_active(stag)) {
            grids.switch(match save {
                SaveGrid::Left  => ltag,
                SaveGrid::Right => rtag,
                SaveGrid::Dont  => ltag,
            });
        }
    }

    pub fn remove(&mut self, tag: u64, rule: ResizeRule) {
        if tag != 0 && !self.grids.is_active(tag) {
            let Screen { ref mut grid_hierarchy, ref mut grids, .. } = *self;
            grid_hierarchy.remove(grids, GridMap::remove, GridMap::resize, tag, rule);
        }
    }

    pub fn switch(&mut self, tag: u64) {
        self.grids.switch(tag);
    }

}

impl Deref for Screen {
    type Target = CharGrid;
    fn deref(&self) -> &CharGrid {
        self.grids.active()
    }
}

impl DerefMut for Screen {
    fn deref_mut(&mut self) -> &mut CharGrid {
        self.grids.active_mut()
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
        &self.grids.find(tag).unwrap()[idx]
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

