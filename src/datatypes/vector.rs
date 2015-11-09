use datatypes::{Coords, Movement, Region};
use datatypes::Movement::*;

/// An iterator representing all coordinates a cursor would pass through in completing some
/// movement within some region.
pub struct Vector {
    pos: Coords,
    mov: Movement,
    len: u32,
    bounds: Region,
}

impl Vector {
    pub fn new(init: Coords, movement: Movement, bounds: Region) -> Vector {
        let (mov, len) = match movement {
            Up(n)       => (Up(1), n),
            Down(n)     => (Down(1), n),
            Left(n)     => (Left(1), n),
            Right(n)    => (Right(1), n),
            UpToEdge    => (Up(1), init.y),
            DownToEdge  => (Down(1), bounds.bottom - init.y),
            LeftToEdge  => (Left(1), init.x),
            RightToEdge => (Right(1), bounds.right - init.x),
            ToBeginning => (Left(1), distance(Coords { x: 0, y: 0 }, init)),
            ToEnd       => (Right(1), distance(init, Coords {x: bounds.right, y: bounds.bottom})),
            _           => unimplemented!(),
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
        self.pos = self.bounds.move_within(self.pos, self.mov);
        self.len = self.len.saturating_sub(1);
        Some(ret)
    }
}

fn distance(from: Coords, to: Coords) -> u32 {
    unimplemented!()
}
