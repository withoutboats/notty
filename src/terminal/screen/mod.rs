use std::collections::HashMap;
use std::mem;
use std::ops::{Deref, DerefMut, Index};

use datatypes::{Coords, CoordsIter, Region};
use terminal::char_grid::{CharGrid, CharCell};

mod grid_hierarchy;

#[cfg(test)]
mod tests;

use self::grid_hierarchy::{GridHierarchy, SplitKind};
use self::grid_hierarchy::GridHierarchy::*;

pub struct Screen {
    width: u32,
    height: u32,
    active_grid: (u64, CharGrid),
    grid_hierarchy: GridHierarchy,
    grids: HashMap<u64, CharGrid>,
}

impl Screen {
    pub fn new(width: u32, height: u32) -> Screen {
        Screen {
            width: width,
            height: height,
            active_grid: (0, CharGrid::new(width, height, false, false)),
            grid_hierarchy: Grid(0),
            grids: HashMap::new(),
        }
    }

    pub fn split_horizontal(&mut self, row: u32, save_left: bool, tag: u64) {
        let grid = if save_left {
            self.active_grid.1.set_height(row);
            CharGrid::new(self.width, self.height - row, false, false)
        else {
            self.active_grid.1.set_height(self.height - row);
            CharGrid::new(self.width, row, false, false)
        }
        self.split(SplitKind::Horizontal(row), save_left, tag, grid);
    }

    pub fn split_vertical(&mut self, col: u32, save_left: bool, tag: u64) {
        let grid = if save_left {
            self.active_grid.1.set_width(col);
            CharGrid::new(self.width - col, self.height, false, false)
        } else {
            self.active_grid.1.set_width(self.width - col);
            CharGrid::new(col, self.height, false, false)
        }
        self.split(SplitKind::Vertical(col), save_left, tag, grid);
    }

    pub fn switch(&mut self, tag: u64) {
        if let Some(grid) = self.grids.remove(&tag) {
            let (tag, grid) = mem::replace(&mut self.active_grid, (tag, grid));
            self.grids.insert(tag, grid);
        }
    }

    pub fn remove(&mut self, tag: u64) {
        if tag != self.active_grid.0 {
            self.grid_hierarchy.remove(tag);
            self.grids.remove(&tag);
        }
    }

    fn split(&mut self, kind: SplitKind, save_left: bool, tag: u64, grid: CharGrid) {
        self.grid_hierarchy.replace(self.active_grid.0, if save_left {
            Split {
                kind: kind,
                left: Box::new(Grid(self.active_grid.0)),
                right: Box::new(Grid(tag)),
            }
        } else {
            Split {
                kind: kind,
                left: Box::new(Grid(tag)),
                right: Box::new(Grid(self.active_grid.0)),
            }
        });
        if save_left {
            self.grids.insert(tag, grid);
        } else {
            let (tag, grid) = mem::replace(&mut self.active_grid, (tag, grid));
            self.grids.insert(tag, grid);
        }
    }
}

impl Deref for Screen {
    type Target = CharGrid;
    fn deref(&self) -> &CharGrid {
        &self.active_grid.1
    }
}

impl DerefMut for Screen {
    fn deref_mut(&mut self) -> &mut CharGrid {
        &mut self.active_grid.1
    }
}

impl Index<Coords> for Screen {
    type Output = CharCell;
    fn index(&self, idx: Coords) -> &CharCell {
        fn index_grid_tree(grid_tree: &GridHierarchy, idx: Coords) -> (u64, Coords) {
            match *grid_tree {
                Grid(key) => (key, idx),
                Split { kind: SplitKind::Horizontal(n), ref left, .. } if idx.y < n => {
                    index_grid_tree(left, idx)
                }
                Split { kind: SplitKind::Horizontal(n), ref right, .. } if idx.y >= n => {
                    index_grid_tree(right, Coords{y: idx.y - n, ..idx})
                }
                Split { kind: SplitKind::Vertical(n), ref left, .. } if idx.x < n => {
                    index_grid_tree(left, idx)
                }
                Split { kind: SplitKind::Vertical(n), ref right, .. } if idx.x >= n => {
                    index_grid_tree(right, Coords{x: idx.x - n, ..idx})
                }
                _ => unreachable!()
            }
        }
        let (key, idx) = index_grid_tree(&self.grid_hierarchy, idx);
        &self.grids.get(&key).or(Some(&self.active_grid.1)).unwrap()[idx]
    }
}

impl<'a> IntoIterator for &'a Screen {
    type Item = &'a CharCell;
    type IntoIter = ScreenIter<'a>;
    fn into_iter(self) -> ScreenIter<'a> {
        ScreenIter {
            screen: self,
            iter: CoordsIter::from_region(Region::new(0, 0, self.width, self.height)),
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
