use std::collections::HashMap;
use std::mem;

use terminal::CharGrid;
use datatypes::Region;

pub struct GridMap {
    active_tag: u64,
    active_grid: CharGrid,
    grids: HashMap<u64, CharGrid>
}

impl GridMap {
    pub fn new(width: u32, height: u32) -> GridMap {
        GridMap {
            active_tag: 0,
            active_grid: CharGrid::new(width, height, false, false),
            grids: HashMap::new(),
        }
    }

    pub fn is_active(&self, tag: u64) -> bool {
        self.active_tag == tag
    }

    pub fn insert(&mut self, tag: u64, width: u32, height: u32) {
        self.grids.insert(tag, CharGrid::new(width, height, false, false));
    }

    pub fn remove(&mut self, tag: u64) {
        self.grids.remove(&tag);
    }

    pub fn resize(&mut self, tag: u64, region: Region) {
        self.find_mut(tag).unwrap().resize(region);
    }

    pub fn switch(&mut self, tag: u64) {
        let GridMap { ref mut active_tag, ref mut active_grid, ref mut grids } = *self;
        grids.get_mut(&tag).map(|grid| {
            mem::swap(active_grid, grid);
            *active_tag = tag;
        });
    }

    pub fn find(&self, tag: u64) -> Option<&CharGrid> {
        if tag == self.active_tag {
            Some(self.active())
        } else {
            self.grids.get(&tag)
        }
    }

    pub fn find_mut(&mut self, tag: u64) -> Option<&mut CharGrid> {
        if tag == self.active_tag {
            Some(self.active_mut())
        } else {
            self.grids.get_mut(&tag)
        }
    }

    pub fn active(&self) -> &CharGrid {
        &self.active_grid
    }

    pub fn active_mut(&mut self) -> &mut CharGrid {
        &mut self.active_grid
    }

    pub fn active_tag(&self) -> u64 { self.active_tag }

}
