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
use self::grid_hierarchy::SplitKind::*;

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

    pub fn stack(&mut self, save: SaveGrid, btag: u64, ttag: u64) {
        let grid = CharGrid::new(self.active_grid.1.grid_width, self.active_grid.1.grid_height,
                                 false, false);
        match save {
            SaveGrid::Left | SaveGrid::Right => {
                let (tag, grid) = mem::replace(&mut self.active_grid, (ttag, grid));
                self.grid_hierarchy.replace(tag, |grid| Stack {
                    tag: tag,
                    top: 1,
                    stack: vec![grid.clone_with_tag(btag), Grid(ttag)]
                });
                self.grids.insert(btag, grid);
            }
            SaveGrid::Dont  => unimplemented!()
        }

    }

    pub fn split_horizontal(&mut self, row: u32, save: SaveGrid, ltag: u64, rtag: u64) {
        let grid = match save {
            SaveGrid::Left  => {
                let height = self.active_grid.1.grid_height - row;
                self.active_grid.1.set_height(row);
                CharGrid::new(self.active_grid.1.grid_width, height, false, false)
            }
            SaveGrid::Right => {
                let height = self.active_grid.1.grid_height - row;
                self.active_grid.1.set_height(height);
                CharGrid::new(self.active_grid.1.grid_width, row, false, false)
            }
            SaveGrid::Dont  => unimplemented!()
        };
        self.split(SplitKind::Horizontal(row), save, ltag, rtag, grid);
    }

    pub fn split_vertical(&mut self, col: u32, save: SaveGrid, ltag: u64, rtag: u64) {
        let grid = match save {
            SaveGrid::Left  => {
                let width = self.active_grid.1.grid_width - col;
                self.active_grid.1.set_width(col);
                CharGrid::new(width, self.active_grid.1.grid_height, false, false)
            }
            SaveGrid::Right => {
                let width = self.active_grid.1.grid_width - col;
                self.active_grid.1.set_width(width);
                CharGrid::new(col, self.active_grid.1.grid_height, false, false)
            }
            SaveGrid::Dont  => unimplemented!()
        };
        self.split(Vertical(col), save, ltag, rtag, grid);
    }

    pub fn switch(&mut self, tag: u64) {
        if let Some(tag) = self.grid_hierarchy.find_first_grid(tag) {
            if let Some(grid) = self.grids.remove(&tag) {
                let (tag, grid) = mem::replace(&mut self.active_grid, (tag, grid));
                self.grids.insert(tag, grid);
            }
        }
    }

    pub fn remove(&mut self, tag: u64) {
        // FIXME when the neighbor is not a grid
        if tag != 0 {
            if let Some((neighbor, split)) = self.grid_hierarchy.remove(tag) {
                let mut n = match split {
                    Horizontal(_)   =>
                        self.grids.get(&tag).unwrap_or(&self.active_grid.1).grid_height,
                    Vertical(_)     =>
                        self.grids.get(&tag).unwrap_or(&self.active_grid.1).grid_width,
                };
                {
                    let grid = self.grids.get_mut(&neighbor).unwrap_or(&mut self.active_grid.1);
                    match split {
                        Horizontal(_)    => {
                            n += grid.grid_height;
                            grid.set_height(n);
                        }
                        Vertical(_)      => {
                            n += grid.grid_width;
                            grid.set_width(n);
                        }
                    }
                }
                if tag == self.active_grid.0 {
                    self.switch(neighbor);
                    self.grids.remove(&tag);
                } else {
                    self.grids.remove(&tag);
                }
            }
        }
    }

    fn split(&mut self, kind: SplitKind, save: SaveGrid, ltag: u64, rtag: u64, grid: CharGrid) {
        let tag = self.active_grid.0;
        self.grid_hierarchy.replace(self.active_grid.0, |grid| match save {
            SaveGrid::Left  => Split {
                tag: tag,
                kind: kind,
                left: Box::new(grid.clone_with_tag(ltag)),
                right: Box::new(Grid(rtag)),
            },
            SaveGrid::Right => Split {
                tag: tag,
                kind: kind,
                left: Box::new(Grid(ltag)),
                right: Box::new(grid.clone_with_tag(rtag)),
            },
            SaveGrid::Dont  => unimplemented!(),
        });
        match save {
            SaveGrid::Left  => {
                self.active_grid.0 = ltag;
                self.grids.insert(rtag, grid);
            }
            SaveGrid::Right => {
                self.active_grid.0 = rtag;
                let (tag, grid) = mem::replace(&mut self.active_grid, (ltag, grid));
                self.grids.insert(tag, grid);
            }
            SaveGrid::Dont  => unimplemented!()
        };
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
                Split { kind: Horizontal(n), ref left, .. } if idx.y < n => {
                    index_grid_tree(left, idx)
                }
                Split { kind: Horizontal(n), ref right, .. } if idx.y >= n => {
                    index_grid_tree(right, Coords{y: idx.y - n, ..idx})
                }
                Split { kind: Vertical(n), ref left, .. } if idx.x < n => {
                    index_grid_tree(left, idx)
                }
                Split { kind: Vertical(n), ref right, .. } if idx.x >= n => {
                    index_grid_tree(right, Coords{x: idx.x - n, ..idx})
                }
                Stack { top, ref stack, .. } => index_grid_tree(&stack[top], idx),
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

pub enum SaveGrid {
    Left, Right, Dont
}
