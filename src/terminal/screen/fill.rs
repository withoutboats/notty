use datatypes::GridSettings;
use terminal::char_grid::CharGrid;
use terminal::image::Image as ImageData;
use terminal::interfaces::{Resizeable, ConstructGrid};

use self::Fill::*;

pub enum Fill {
    Grid(CharGrid),
    Image(ImageData),
}

impl Fill {
    pub fn grid(&self) -> Option<&CharGrid> {
        match *self {
            Grid(ref grid)  => Some(grid),
            _               => None
        }
    }

    pub fn grid_mut(&mut self) -> Option<&mut CharGrid> {
        match *self {
            Grid(ref mut grid)  => Some(grid),
            _                   => None
        }
    }
}

impl Resizeable for Fill {
    fn dims(&self) -> (u32, u32) {
        match *self {
            Grid(ref grid)      => grid.dims(),
            Image(ref image)    => image.dims(),
        }
    }

    fn resize_width(&mut self, width: u32) {
        match *self {
            Grid(ref mut grid)      => grid.resize_width(width),
            Image(ref mut image)    => image.resize_width(width),
        }
    }

    fn resize_height(&mut self, height: u32) {
        match *self {
            Grid(ref mut grid)      => grid.resize_height(height),
            Image(ref mut image)    => image.resize_height(height),
        }
    }
}

impl ConstructGrid for Fill {
    fn new(settings: GridSettings) -> Fill {
        Fill::Grid(CharGrid::new(settings))
    }
}
