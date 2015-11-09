use datatypes::{Coords, Direction, Movement, Region};
use datatypes::Direction::*;
use datatypes::Movement::*;

/// An iterator representing all coordinates a cursor would pass through in completing some
/// movement within some region.
pub struct Vector {
    pos: Coords,
    mov: Direction,
    len: u32,
    bounds: Region,
}

impl Vector {
    pub fn new(init: Coords, movement: Movement, bounds: Region) -> Vector {
        let (mov, len) = match movement {
            To(d, n)        => (d, n),
            ToEdge(Up)      => (Up, init.y),
            ToEdge(Down)    => (Down, bounds.bottom - init.y),
            ToEdge(Left)    => (Left, init.x),
            ToEdge(Right)   => (Right, bounds.right - init.x),
            ToBeginning     => (Left, distance(Coords { x: 0, y: 0 }, init)),
            ToEnd           => (Right, distance(init, Coords {x: bounds.right, y: bounds.bottom})),
            _               => unimplemented!(),
        };
        Vector {
            pos: init,
            mov: mov,
            len: len,
            bounds: bounds,
        }
    }
}

impl Iterator for Vector {
    type Item = Coords;

    fn next(&mut self) -> Option<Coords> {
        if self.len == 0 { return None; }
        let ret = self.pos;
        self.pos = self.bounds.move_within(self.pos, To(self.mov, 1));
        self.len = self.len.saturating_sub(1);
        Some(ret)
    }
}

fn distance(from: Coords, to: Coords) -> u32 {
    unimplemented!()
}
