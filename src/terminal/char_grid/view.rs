use datatypes::{Coords, Region, GridSettings, Flow};
use terminal::interfaces::{ConstructGrid, Resizeable};

use self::View::*;

#[derive(Eq, PartialEq, Debug)]
pub enum View {
    Moveable(Region),
    Reflowable(ReflowableView),
}

impl ConstructGrid for View {
    fn new(settings: GridSettings) -> View {
        match settings.flow {
            Flow::Moveable      => View::Moveable(Region::new(0, 0, settings.width, settings.height)),
            Flow::Reflowable    => unimplemented!()
        }
    }
}

impl Resizeable for View {
    fn resize_width(&mut self, width: u32) {
        match *self {
            Moveable(ref mut region)    => region.right = region.left + width,
            Reflowable(_)               => unimplemented!()
        }
    }

    fn resize_height(&mut self, height: u32) {
        match *self {
            Moveable(ref mut region)    => region.bottom = region.top + height,
            Reflowable(_)               => unimplemented!()
        }
    }
}

impl View {
    pub fn translate(&self, Coords { x, y }: Coords) -> Coords {
        match *self {
            Moveable(region)    => {
                let coords = Coords { x: x + region.left, y: y + region.top };
                assert!(region.contains(coords));
                coords
            }
            Reflowable(_)       => unimplemented!()
        }
    }

    pub fn width(&self) -> u32 {
        match *self {
            Moveable(region)    => region.width(),
            Reflowable(_)       => unimplemented!()
        }
    }

    pub fn height(&self) -> u32 {
        match *self {
            Moveable(region)    => region.height(),
            Reflowable(_)       => unimplemented!()
        }
    }

    pub fn bounds(&self) -> Region {
        match *self {
            Moveable(region)    => region,
            Reflowable(_)       => unimplemented!()
        }
    }

    pub fn keep_within(&mut self, coords: Coords) {
        match *self {
            Moveable(ref mut region) => {
                *region = region.move_to_contain(coords);
            }
            Reflowable(_) => {
                unimplemented!()
            }
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct ReflowableView {
    point: Coords,
    width: u32,
    height: u32,
    line_wraps: Vec<u32>,
}

// impl ReflowableView {
//     fn keep_cursor_within(&mut self, coords: Coords, grid: &CharGrid) {
//         if (/*coords is below point*/) {
//             self.update_line_wraps(coords, grid);
//             //possibly adjust point down
//         }
//     }
// 
//     fn update_line_wraps(&mut self, grid: &CharGrid) {
//         let mut line_wraps_sum = 0;
//         for i in 0..self.height {
//             let coords = Coords { x: self.point.x + self.width, y: self.point.y + i };
//             let line_wrap_count = count_wraps(grid, coords, self.width);
//             if let Some(value) = self.line_wraps.get_mut(i) {
//                 *value = line_wrap_counts;
//             } else {
//                 self.line_wraps.push(line_wrap_counts)
//             }
//             line_wraps_sum += line_wrap_count;
//         }
//     }
// }
// 
// fn count_wraps(grid: &CharGrid, coords: Coords, width: u32) -> u32 {
//     let cells_with_content = grid.row_from(coords)
//                                  .map(|cell| !cell.is_empty())
//                                  .enumerate()
//                                  .select(|&(_, x)| x)
//                                  .last().unwrap_or(0);
//     cells_with_content / width
// }
