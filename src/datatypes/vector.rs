use datatypes::{Coords, Direction, Movement, Region};
use datatypes::Direction::*;
use datatypes::Movement::*;

/// An iterator representing all coordinates a cursor would pass through in completing some
/// movement within some region.
pub struct Vector {
    pos: Coords,
    mov: Direction,
    len: u32,
    wrap: bool,
    bounds: Region,
}

impl Vector {
    pub fn new(init: Coords, movement: Movement, bounds: Region) -> Vector {
        let (mov, len, wrap) = match movement {
            To(d, n, wrap)  => (d, n, wrap),
            ToEdge(Up)      => (Up, init.y, false),
            ToEdge(Down)    => (Down, bounds.bottom - init.y, false),
            ToEdge(Left)    => (Left, init.x, false),
            ToEdge(Right)   => (Right, bounds.right - init.x, false),
            ToBeginning     => (Left, distance(Coords { x: 0, y: 0 }, init), false),
            ToEnd           => (Right, distance(init, Coords {x: bounds.right, y: bounds.bottom}), false),
            _               => unimplemented!(),
        };
        Vector {
            pos: init,
            mov: mov,
            len: len,
            wrap: wrap,
            bounds: bounds,
        }
    }
}

impl Iterator for Vector {
    type Item = Coords;

    fn next(&mut self) -> Option<Coords> {
        if self.len == 0 { return None; }
        let ret = self.pos;
        self.pos = self.bounds.move_within(self.pos, To(self.mov, 1, self.wrap));
        self.len = self.len.saturating_sub(1);
        Some(ret)
    }
}

fn distance(from: Coords, to: Coords) -> u32 {
    unimplemented!()
}
