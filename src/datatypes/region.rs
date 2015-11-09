use std::cmp;
use std::iter::IntoIterator;

use cfg;
use datatypes::{Coords, Direction, Movement};
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

    pub fn move_and_scroll(&self, Coords {x, y}: Coords, movement: Movement)
            -> (Coords, u32, Direction) {
        match movement {
            UpIndex(n) if self.top + n > y          => {
                (Coords {x: x, y: self.top}, self.top + n - y, Direction::Up)
            }
            DownIndex(n) if n + y >= self.bottom    => {
                (Coords {x: x, y: n + y}, n + y - self.bottom + 1, Direction::Down)
            }
            LeftIndex(n) if self.left + n > x       => {
                (Coords {x: self.left, y: y}, self.left + n - x, Direction::Left)
            }
            RightIndex(n) if n + x >= self.right    => {
                (Coords {x: self.right - 1, y: y}, n + y - self.bottom + 1, Direction::Right)
            }
            PreviousLine(n) if self.top + n > y     => {
                (Coords {x: 0, y: self.top}, self.top + n - y, Direction::Up)
            }
            NextLine(n) if n + y >= self.bottom     => {
                (Coords {x: 0, y: n + y}, n + y - self.bottom + 1, Direction::Down)
            }
            _                                       => {
                (self.move_within(Coords {x: x, y: y}, movement), 0, Direction::Down)
            }
        }
    }

    /// Calculate the movement from one coordinate to another within this region.
    pub fn move_within(&self, Coords {x, y}: Coords, movement: Movement) -> Coords {
        match movement {
            Position(coords)            => self.xy_within(coords),
            Column(n)                   => Coords {x: self.x_within(n), y: y},
            Row(n)                      => Coords {x: x, y: self.y_within(n)},
            UpToEdge                    => Coords {x: x, y: self.top},
            DownToEdge                  => Coords {x: x, y: self.bottom - 1},
            LeftToEdge                  => Coords {x: self.left, y: y},
            RightToEdge                 => Coords {x: self.right - 1, y: y},
            ToBeginning                 => Coords {x: self.left, y: self.top},
            ToEnd                       => Coords {x: self.right - 1, y: self.bottom - 1},
            Up(n) | UpIndex(n)          => {
                Coords {x: x, y: cmp::max(self.top, y.saturating_sub(n))}
            }
            Down(n) | DownIndex(n)      => {
                Coords {x: x, y: cmp::min(y.saturating_add(n), self.bottom - 1)}
            }
            Left(n) | LeftIndex(n)      => {
                Coords {x: cmp::max(self.left, x.saturating_sub(n)), y: y}
            }
            Right(n) | RightIndex(n)    => {
                Coords {x: cmp::min(x.saturating_add(n), self.right - 1), y: y}
            }
            LeftTab(n)                  => {
                let tab = ((x / cfg::TAB_STOP).saturating_sub(n)) * cfg::TAB_STOP;
                Coords {x: cmp::max(tab, self.left), y: y}
            }
            RightTab(n)                 => {
                let tab = ((x / cfg::TAB_STOP) + n) * cfg::TAB_STOP;
                Coords {x: cmp::min(tab, self.right - 1), y: y}
            }
            PreviousLine(n)             => {
                Coords {x: 0, y: cmp::max(y.saturating_sub(n), self.top)}
            }
            NextLine(n)                 => {
                Coords {x: 0, y: cmp::min(y.saturating_add(n), self.bottom - 1)}
            }
        }
    }

    /// Iterate over the coordinates in the region, starting in the upper left and moving rightward
    /// and wrapping at the right edge of the region.
    pub fn iter(&self) -> RegionIter {
        RegionIter {
            region: *self,
            point: Coords {x: self.left, y: self.top},
            back_point: Coords {x: self.left, y: self.bottom},
        }
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
    type IntoIter = RegionIter;

    fn into_iter(self) -> RegionIter {
        self.iter()
    }
}

pub struct RegionIter {
    region: Region,
    point: Coords,
    back_point: Coords,
}

impl Iterator for RegionIter {
    type Item = Coords;

    fn next(&mut self) -> Option<Coords> {
        if self.point != self.back_point {
            let point = self.point;
            self.point = if point.x == self.region.right - 1 {
                Coords {x: self.region.left, y: point.y + 1}
            } else { Coords {x: point.x + 1, y: point.y} };
            Some(point)
        } else { None }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

}

impl DoubleEndedIterator for RegionIter {
    fn next_back(&mut self) -> Option<Coords> {
        if self.point != self.back_point {
            self.back_point = if self.back_point.x == self.region.left {
                Coords {x: self.region.right - 1, y: self.back_point.y - 1}
            } else { Coords {x: self.back_point.x - 1, y: self.back_point.y} };
            Some(self.back_point)
        } else { None }
    }
}

impl ExactSizeIterator for RegionIter {
    fn len(&self) -> usize {
        if self.point.y == self.back_point.y {
            (self.back_point.x  - self.point.x) as usize
        } else {
            let width  = self.region.right - self.region.left;
            let first  = self.region.right - self.point.x;
            let middle = (self.back_point.y - self.point.y - 1) * width;
            let last   = self.back_point.y - self.region.left;
            (first + middle + last) as usize
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use datatypes::{Coords, Movement};
    use datatypes::Movement::*;

    static REGION: &'static Region = &Region { left: 0, top: 10, right: 100, bottom: 100 }; 

    static COORDS: &'static [(Coords, bool)] = &[
        (Coords { x: 0, y: 0 }, false),
        (Coords { x: 0, y: 10 }, true),
        (Coords { x: 50, y: 50 }, true),
        (Coords { x: 99, y: 99 }, true),
        (Coords { x: 100, y: 0 }, false),
        (Coords { x: 100, y: 100 }, false),
        (Coords { x: 200, y: 200 }, false),
    ];

    static MOVEMENTS: &'static [(Coords, Movement, bool, Coords)] = &[
        (Coords { x:50, y:50 }, Position(Coords { x:40, y:40 }), true, Coords { x:40, y:40 }),
        (Coords { x:50, y:50 }, Position(Coords { x:200, y:200 }), true, Coords { x:99, y:99}),
        (Coords { x:50, y:50 }, Position(Coords { x:0, y:0 }), true, Coords { x:0, y:10 }),
        (Coords { x:50, y:50 }, Column(0), true, Coords { x:0, y:50 }),
        (Coords { x:50, y:50 }, Column(10), true, Coords { x:10, y:50 }),
        (Coords { x:50, y:50 }, Column(100), true, Coords { x:99, y:50 }),
        (Coords { x:50, y:50 }, Row(0), true, Coords { x:50, y:10 }),
        (Coords { x:50, y:50 }, Row(10), true, Coords { x:50, y:10 }),
        (Coords { x:50, y:50 }, Row(100), true, Coords { x:50, y:99 }),
        (Coords { x:50, y:50 }, UpToEdge, true, Coords { x:50, y:10 }),
        (Coords { x:50, y:50 }, DownToEdge, true, Coords { x:50, y:99 }),
        (Coords { x:50, y:50 }, LeftToEdge, true, Coords { x:0, y:50 }),
        (Coords { x:50, y:50 }, RightToEdge, true, Coords { x:99, y:50 }),
        (Coords { x:50, y:50 }, ToBeginning, true, Coords { x:0, y:10 }),
        (Coords { x:50, y:50 }, ToEnd, true, Coords { x:99, y:99 }),
        (Coords { x:50, y:50 }, Up(5), true, Coords { x:50, y:45 }),
        (Coords { x:50, y:50 }, Down(5), true, Coords { x:50, y:55 }),
        (Coords { x:50, y:50 }, Left(5), false, Coords { x:45, y:50 }),
        (Coords { x:50, y:50 }, Right(5), false, Coords { x:55, y:50 }),
        (Coords { x:50, y:50 }, PreviousLine(1), true, Coords { x:0, y:49 }),
        (Coords { x:50, y:50 }, NextLine(1), true, Coords { x:0, y:51 }),
    ];

    #[test]
    fn region_contains_coords() {
        for &(coords, b) in COORDS {
            assert!(REGION.contains(coords) == b, "{:?}", coords);
        }
    }

    #[test]
    fn region_calculates_movements() {
        for &(from, moves, wrapping, to) in MOVEMENTS {
            assert!(REGION.move_within(from, moves) == to,
                    "{:?} {:?} {} {:?}", from, moves, wrapping, to);
        }
    }

    #[test]
    fn region_iterates() {
        let coords = (10..100).flat_map(|y| (0..100).map(move |x| Coords {x: x, y: y}));
        REGION.iter().zip(coords).all(|(c1, c2)| { assert_eq!(c1, c2); true });
    }

}
