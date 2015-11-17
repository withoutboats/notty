//  notty is a new kind of terminal emulator.
//  Copyright (C) 2015 without boats
//  
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//  
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU Affero General Public License for more details.
//  
//  You should have received a copy of the GNU Affero General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
use std::mem;

use datatypes::{Area, Coords, Direction, Region, move_within};
use datatypes::Area::*;
use datatypes::Direction::*;
use datatypes::Movement::To;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct CoordsIter {
    point: Coords,
    back_point: Coords,
    region: Region,
    dir: Direction,
    fin: bool,
}

impl CoordsIter {

    pub fn from_area(area: Area, cursor: Coords, screen: Region) -> CoordsIter {
        match area {
            CursorCell              => CoordsIter {
                point: cursor,
                back_point: cursor,
                region: screen,
                dir: Right,
                fin: false
            },
            CursorRow               => CoordsIter {
                point: Coords {x: screen.left, y: cursor.y},
                back_point: Coords {x: screen.right - 1, y: cursor.y},
                region: screen,
                dir: Right,
                fin: false,
            },
            CursorColumn            => CoordsIter {
                point: Coords {x: cursor.x, y: screen.top},
                back_point: Coords {x: cursor.x, y: screen.bottom - 1},
                region: screen,
                dir: Down,
                fin: false,
            },
            CursorTo(mov)           => CoordsIter {
                point: cursor,
                back_point: move_within(cursor, mov, screen),
                region: screen,
                dir: mov.direction(cursor),
                fin: false,
            },
            CursorBound(coords) if coords == cursor => {
                CoordsIter::from_area(CursorCell, cursor, screen)
            }
            CursorBound(coords)     => {
                CoordsIter::from_region(Region::new(cursor.x, cursor.y, coords.x, coords.y))
            }
            WholeScreen             => CoordsIter::from_region(screen),
            Bound(region)           => CoordsIter::from_region(region),
            Rows(top, bottom)       => CoordsIter {
                point: Coords {x: screen.left, y: top },
                back_point: Coords {x: screen.right - 1, y: bottom - 1},
                region: screen,
                dir: Right,
                fin: !(top < bottom),
            },
            Columns(left, right)    => CoordsIter {
                point: Coords {x: left, y: screen.top},
                back_point: Coords {x: right - 1, y: screen.bottom - 1},
                region: screen,
                dir: Down,
                fin: !(left < right),
            },
            BelowCursor(true)       => {
                CoordsIter::from_region(Region { top: cursor.y, ..screen})
            }
            BelowCursor(false) if cursor.y == screen.bottom - 1 => {
                CoordsIter {
                    point: cursor,
                    back_point: cursor,
                    region: screen,
                    dir: Right,
                    fin: true,
                }
            }
            BelowCursor(false)       => {
                CoordsIter::from_region(Region { top: cursor.y + 1, ..screen })
            }
        }
    }

    pub fn from_region(region: Region) -> CoordsIter {
        CoordsIter {
            point: Coords {x: region.left, y: region.top},
            back_point: Coords {x: region.right-1, y: region.bottom-1},
            region: region,
            dir: Right,
            fin: false,
        }
    }

}

impl Iterator for CoordsIter {
    type Item = Coords;

    fn next(&mut self) -> Option<Coords> {
        match (self.point == self.back_point, self.fin) {
            (_, true)   => None,
            (true, _)   => {
                self.fin = true;
                Some(self.point)
            }
            (false, _)  => {
                let point = move_within(self.point, To(self.dir, 1, true), self.region);
                Some(mem::replace(&mut self.point, point))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }

}

impl DoubleEndedIterator for CoordsIter {

    fn next_back(&mut self) -> Option<Coords> {
        match (self.point == self.back_point, self.fin) {
            (_, true)   => None,
            (true, _)   => {
                self.fin = true;
                Some(self.point)
            }
            (false, _)  => {
                let point = move_within(self.back_point, To(self.dir.rev(), 1, true), self.region);
                Some(mem::replace(&mut self.back_point, point))
            }
        }
    }
}

impl ExactSizeIterator for CoordsIter {

    fn len(&self) -> usize {

        match self.dir {
            Up if self.point.x == self.back_point.x     => {
                (self.point.y - self.back_point.y + 1) as usize
            }
            Up                                          => {
                let height = self.region.bottom - self.region.top;
                let first = self.point.y - self.region.top + 1;
                let mid = (self.point.x - self.back_point.x).saturating_sub(1) * height;
                let last = self.region.bottom - self.back_point.y;
                (first + mid + last) as usize
            }
            Down if self.point.x == self.back_point.x   => {
                (self.back_point.y - self.point.y + 1) as usize
            }
            Down                                        => {
                let height = self.region.bottom - self.region.top;
                let first = self.region.bottom - self.point.y;
                let mid = (self.back_point.x - self.point.x).saturating_sub(1) * height;
                let last = self.back_point.y - self.region.top + 1;
                (first + mid + last) as usize
            }
            Left if self.point.y == self.back_point.y   => {
                (self.point.x - self.back_point.x + 1) as usize
            }
            Left                                        => {
                let width = self.region.right - self.region.left;
                let first = self.point.x - self.region.left + 1;
                let mid = (self.point.y - self.back_point.y).saturating_sub(1) * width;
                let last = self.region.right - self.back_point.x;
                (first + mid + last) as usize
            }
            Right if self.point.y == self.back_point.y  => {
                (self.back_point.x - self.point.x + 1)  as usize
            }
            Right                                       => {
                let width = self.region.right - self.region.left;
                let first = self.region.right - self.point.x;
                let mid = (self.back_point.y - self.point.y).saturating_sub(1) * width;
                let last = self.back_point.x - self.region.left + 1;
                (first + mid + last) as usize
            }
        }
    }

}

#[cfg(test)]
mod tests {

    use super::*;
    use datatypes::{Coords, Region};
    use datatypes::Direction::*;

    macro_rules! iter {
        ($x1:expr, $y1:expr, $x2:expr, $y2:expr, $l:expr, $t: expr, $r:expr, $b:expr, $d:expr) =>
        (CoordsIter {
            point: Coords {x: $x1, y: $y1},
            back_point: Coords {x: $x2, y: $y2},
            region: Region {left: $l, top: $t, right: $r, bottom: $b},
            dir: $d,
            fin: false,
        });
    }

    macro_rules! array {
        [$(($x:expr,$y:expr)),*] => [
            &[$(Coords {x: $x, y: $y }),*]
        ];
    }

    static TEST_CASES: &'static [(CoordsIter, &'static [Coords])] = &[
        (iter!(1,1, 0,0, 0,0,2,2, Up),      array![(1,1), (1,0), (0,1), (0,0)]),
        (iter!(0,1, 0,0, 0,0,2,2, Up),      array![(0,1), (0,0)]),
        (iter!(1,0, 0,1, 0,0,2,2, Up),      array![(1,0), (0,1)]),
        (iter!(0,0, 1,1, 0,0,2,2, Down),    array![(0,0), (0,1), (1,0), (1,1)]),
        (iter!(0,0, 0,1, 0,0,2,2, Down),    array![(0,0), (0,1)]),
        (iter!(0,1, 1,0, 0,0,2,2, Down),    array![(0,1), (1,0)]),
        (iter!(1,1, 0,0, 0,0,2,2, Left),    array![(1,1), (0,1), (1,0), (0,0)]),
        (iter!(1,0, 0,0, 0,0,2,2, Left),    array![(1,0), (0,0)]),
        (iter!(0,1, 1,0, 0,0,2,2, Left),    array![(0,1), (1,0)]),
        (iter!(0,0, 1,1, 0,0,2,2, Right),   array![(0,0), (1,0), (0,1), (1,1)]),
        (iter!(0,0, 1,0, 0,0,2,2, Right),   array![(0,0), (1,0)]),
        (iter!(1,0, 0,1, 0,0,2,2, Right),   array![(1,0), (0,1)]),
    ];

    #[test]
    fn forward_iteration() {
        for &(iter, array) in TEST_CASES {
            assert_eq!(iter.collect::<Vec<_>>(), array.iter().cloned().collect::<Vec<_>>());
        }
    }

    #[test]
    fn backward_iteration() {
        for &(iter, array) in TEST_CASES {
            assert_eq!(iter.rev().collect::<Vec<_>>(),
                       array.iter().cloned().rev().collect::<Vec<_>>());
        }
    }

    #[test]
    fn length() {
        for &(iter, array) in TEST_CASES {
            assert!(iter.len() == array.len(), "{:?}; {} != {}", iter, iter.len(), array.len());
        }
    }
}
