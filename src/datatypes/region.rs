use std::cmp;
use std::iter::IntoIterator;

use cfg;
use datatypes::{Coords, CoordsIter, Movement};
use datatypes::Direction::*;
use datatypes::Movement::*;

/// A concrete, rectangular region of the screen.
///
/// The region is incluse of the top and left boundary and exclusive of the bottom and right
/// boundary.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Region {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

impl Region {
    /// Creates a region. Note that x1/x2 and y1/y2 need not be properly ordered, but one of them 
    /// __must__ be greater than the other. This function will panic otherwise.
    pub fn new(x1: u32, y1: u32, x2: u32, y2: u32) -> Region {
        let (left, right) = (cmp::min(x1, x2), cmp::max(x1, x2));
        let (top, bottom) = (cmp::min(y1, y2), cmp::max(y1, y2));
        assert!(right > 0);
        assert!(bottom > 0);
        Region {
            left: left,
            top: top,
            right: right,
            bottom: bottom,
        }
    }

    /// Returns true if a given coordinates is contained within this region.
    pub fn contains(&self, coords: Coords) -> bool {
        self.left <= coords.x && coords.x < self.right
            && self.top <= coords.y && coords.y < self.bottom
    }

    /// Returns the width of this region.
    pub fn width(&self) -> u32 {
        self.right - self.left
    }

    /// Returns the height of this region.
    pub fn height(&self) -> u32 {
        self.bottom - self.top
    }

    /// Calculate the movement from one coordinate to another within this region.
    pub fn move_within(&self, Coords {x, y}: Coords, movement: Movement) -> Coords {
        match movement {
            Position(coords)    => self.xy_within(coords),
            Column(n)           => Coords {x: self.x_within(n), y: y},
            Row(n)              => Coords {x: x, y: self.y_within(n)},
            ToEdge(Up)          => Coords {x: x, y: self.top},
            ToEdge(Down)        => Coords {x: x, y: self.bottom - 1},
            ToEdge(Left)        => Coords {x: self.left, y: y},
            ToEdge(Right)       => Coords {x: self.right - 1, y: y},
            ToBeginning         => Coords {x: self.left, y: self.top},
            ToEnd               => Coords {x: self.right - 1, y: self.bottom - 1},
            To(Up, n, true) if self.top + n > y       => {
                let x = x.saturating_sub((self.top + n - y) / (self.bottom - self.top) + 1);
                let y = self.bottom - (self.top + n - y) % (self.bottom - self.top);
                if x < self.left {
                    Coords { x: self.left, y: self.top }
                } else {
                    Coords { x: x, y: y }
                }
            }
            To(Down, n, true) if y + n >= self.bottom  => {
                let x = x + (y + n - self.bottom) / (self.bottom - self.top) + 1;
                let y = self.top + (y + n - self.bottom) % (self.bottom - self.top);
                if x >= self.right {
                    Coords { x: self.right - 1, y: self.bottom - 1 }
                } else {
                    Coords { x: x, y: y }
                }
            }
            To(Left, n, true) if self.left + n > x    => {
                let y = y.saturating_sub((self.left + n - x) / (self.right - self.left) + 1);
                let x = self.right - (self.left + n - x) % (self.right - self.left);
                if y < self.top {
                    Coords { x: self.left, y: self.top }
                } else {
                    Coords { x: x, y: y }
                }
            }
            To(Right, n, true) if x + n >= self.right  => {
                let y = y + (x + n - self.right) / (self.right - self.left) + 1;
                let x = self.left + (x + n - self.right) % (self.right - self.left);
                if y >= self.bottom {
                    Coords { x: self.right - 1, y: self.bottom - 1 }
                } else {
                    Coords { x: x, y: y }
                }
            }
            To(Up, n, _) | IndexTo(Up, n)         => {
                Coords {x: x, y: cmp::max(self.top, y.saturating_sub(n))}
            }
            To(Down, n, _) | IndexTo(Down, n)     => {
                Coords {x: x, y: cmp::min(y.saturating_add(n), self.bottom - 1)}
            }
            To(Left, n, _) | IndexTo(Left, n)     => {
                Coords {x: cmp::max(self.left, x.saturating_sub(n)), y: y}
            }
            To(Right, n, _) | IndexTo(Right, n)   => {
                Coords {x: cmp::min(x.saturating_add(n), self.right - 1), y: y}
            }
            Tab(Left, n, true) if self.left + n > x => {
                unimplemented!()
            }
            Tab(Right, n, true) if x + n >= self.right => {
                unimplemented!()
            }
            Tab(Left, n, false)                 => {
                let tab = ((x / cfg::TAB_STOP).saturating_sub(n)) * cfg::TAB_STOP;
                Coords {x: cmp::max(tab, self.left), y: y}
            }
            Tab(Right, n, false)                => {
                let tab = ((x / cfg::TAB_STOP) + n) * cfg::TAB_STOP;
                Coords {x: cmp::min(tab, self.right - 1), y: y}
            }
            Tab(..)                             => unimplemented!(),
            PreviousLine(n)                     => {
                Coords {x: 0, y: cmp::max(y.saturating_sub(n), self.top)}
            }
            NextLine(n)                         => {
                Coords {x: 0, y: cmp::min(y.saturating_add(n), self.bottom - 1)}
            }
        }
    }

    /// Iterate over the coordinates in the region, starting in the upper left and moving rightward
    /// and wrapping at the right edge of the region.
    pub fn iter(&self) -> CoordsIter {
        CoordsIter::from_region(*self)
    }

    /// Calculate the nearest coordinate within the region.
    pub fn xy_within(&self, Coords {x, y}: Coords) -> Coords {
        Coords {
            x: self.x_within(x),
            y: self.y_within(y),
        }
    }

    /// Calculate the nearest x value within the region.
    pub fn x_within(&self, x: u32) -> u32 {
        cmp::max(cmp::min(x, self.right - 1), self.left)
    }

    /// Calculate the naerest y value within the region.
    pub fn y_within(&self, y: u32) -> u32 {
        cmp::max(cmp::min(y, self.bottom - 1), self.top)
    }

}

impl IntoIterator for Region {
    type Item = Coords;
    type IntoIter = CoordsIter;

    fn into_iter(self) -> CoordsIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use datatypes::{Coords, Movement};
    use datatypes::Movement::*;
    use datatypes::Direction::*;

    static REGION: Region = Region { left: 0, top: 10, right: 100, bottom: 100 }; 

    static COORDS: &'static [(Coords, bool)] = &[
        (Coords { x: 0, y: 0 }, false),
        (Coords { x: 0, y: 10 }, true),
        (Coords { x: 50, y: 50 }, true),
        (Coords { x: 99, y: 99 }, true),
        (Coords { x: 100, y: 0 }, false),
        (Coords { x: 100, y: 100 }, false),
        (Coords { x: 200, y: 200 }, false),
    ];

    static MOVEMENTS: &'static [(Coords, Movement, Coords)] = &[
        (Coords { x:50, y:50 }, Position(Coords { x:40, y:40 }), Coords { x:40, y:40 }),
        (Coords { x:50, y:50 }, Position(Coords { x:200, y:200 }), Coords { x:99, y:99}),
        (Coords { x:50, y:50 }, Position(Coords { x:0, y:0 }), Coords { x:0, y:10 }),
        (Coords { x:50, y:50 }, Column(0), Coords { x:0, y:50 }),
        (Coords { x:50, y:50 }, Column(10), Coords { x:10, y:50 }),
        (Coords { x:50, y:50 }, Column(100), Coords { x:99, y:50 }),
        (Coords { x:50, y:50 }, Row(0), Coords { x:50, y:10 }),
        (Coords { x:50, y:50 }, Row(10), Coords { x:50, y:10 }),
        (Coords { x:50, y:50 }, Row(100), Coords { x:50, y:99 }),
        (Coords { x:50, y:50 }, ToEdge(Up), Coords { x:50, y:10 }),
        (Coords { x:50, y:50 }, ToEdge(Down), Coords { x:50, y:99 }),
        (Coords { x:50, y:50 }, ToEdge(Left), Coords { x:0, y:50 }),
        (Coords { x:50, y:50 }, ToEdge(Right), Coords { x:99, y:50 }),
        (Coords { x:50, y:50 }, ToBeginning, Coords { x:0, y:10 }),
        (Coords { x:50, y:50 }, ToEnd, Coords { x:99, y:99 }),
        (Coords { x:50, y:50 }, To(Up, 5, false), Coords { x:50, y:45 }),
        (Coords { x:50, y:50 }, To(Down, 5, false), Coords { x:50, y:55 }),
        (Coords { x:50, y:50 }, To(Left, 5, false), Coords { x:45, y:50 }),
        (Coords { x:50, y:50 }, To(Right, 5, false), Coords { x:55, y:50 }),
        (Coords { x:50, y:15 }, To(Up, 10, true), Coords { x: 49, y: 95 }),
        (Coords { x:50, y:15 }, To(Up, 180, true), Coords { x: 48, y: 15 }),
        (Coords { x:50, y:95 }, To(Down, 10, true), Coords { x: 51, y: 15 }),
        (Coords { x:50, y:95 }, To(Down, 180, true), Coords { x: 52, y: 95 }),
        (Coords { x:5,  y:50 }, To(Left, 10, true), Coords { x: 95, y: 49 }),
        (Coords { x:5,  y:50 }, To(Left, 200, true), Coords { x: 5, y: 48 }),
        (Coords { x:95, y:50 }, To(Right, 10, true), Coords { x: 5, y: 51 }),
        (Coords { x:95, y:50 }, To(Right, 200, true), Coords { x: 95, y: 52 }),
        (Coords { x:50, y:50 }, PreviousLine(1), Coords { x:0, y:49 }),
        (Coords { x:50, y:50 }, NextLine(1), Coords { x:0, y:51 }),
    ];

    #[test]
    fn region_contains_coords() {
        for &(coords, b) in COORDS {
            assert!(REGION.contains(coords) == b, "{:?}", coords);
        }
    }

    #[test]
    fn region_calculates_movements() {
        for &(from, moves, to) in MOVEMENTS {
            let result = REGION.move_within(from, moves);
            assert!(result == to, "{:?} {:?} : {:?} != {:?}", from, moves, result, to);
        }
    }

    #[test]
    fn region_iterates() {
        let coords = (10..100).flat_map(|y| (0..100).map(move |x| Coords {x: x, y: y}));
        REGION.iter().zip(coords).all(|(c1, c2)| { assert_eq!(c1, c2); true });
    }

}
