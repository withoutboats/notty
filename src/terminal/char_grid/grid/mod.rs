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
use std::cmp;
use std::collections::VecDeque;
use std::mem;
use std::sync::atomic::Ordering::Relaxed;

use cfg::SCROLLBACK;
use datatypes::{Coords, Direction, Region, GridSettings};
use terminal::interfaces::{ConstructGrid, CellGrid, Resizeable, WriteableCell, WriteableGrid, Cell};

pub struct Grid<T> {
    pub width: usize,
    pub height: usize,
    data: VecDeque<T>,
    rem_x: Option<usize>,
    rem_y: Option<usize>,
}

impl<T> ConstructGrid for Grid<T> {
    fn new(settings: GridSettings) -> Grid<T> {
        match (settings.retain_offscreen_state, SCROLLBACK.load(Relaxed)) {
            (false, _)          => Grid::with_caps(settings.width as usize, settings.height as usize),
            (_, n) if n > 0     => Grid::with_caps(cmp::min(n as usize, settings.width as usize),
                                                   cmp::min(n as usize, settings.height as usize)),
            _                   => Grid::with_infinite_scroll(),
        }
    }
}

impl<T: Cell + Default> CellGrid for Grid<T> {
    type Cell = T;

    fn get(&self, coords: Coords) -> Option<&T> {
        self.bounds().and_then(move |bounds| if bounds.contains(coords) { 
            Some(&self.data[linearize(self.width, coords)])
        } else { None })
    }

    fn get_mut(&mut self, coords: Coords) -> Option<&mut T> {
        self.bounds().and_then(move |bounds| if bounds.contains(coords) { 
            Some(&mut self.data[linearize(self.width, coords)])
        } else { None })
    }

    fn moveover(&mut self, from: Coords, to: Coords) {
        if let Some(from) = self.get_mut(from).map(|cell| mem::replace(cell, T::default())) {
            self.fill_to(to);
            *self.get_mut(to).unwrap() = from;
        }
    }

    fn move_out_of_extension(&self, mut coords: Coords, direction: Direction) -> Coords {
        fn up(Coords { x, y }: Coords) -> Coords    { Coords { x: x, y: y - 1 } }
        fn down(Coords { x, y }: Coords) -> Coords  { Coords { x: x, y: y + 1 } }
        fn left(Coords { x, y }: Coords) -> Coords  { Coords { x: x - 1, y: y } }
        fn right(Coords { x, y }: Coords) -> Coords { Coords { x: x + 1, y: y } }

        loop {
            match self.get(coords).map(T::is_extension) {
                Some(true)  => coords = match direction {
                    Direction::Up       => up(coords),
                    Direction::Down     => down(coords),
                    Direction::Left     => left(coords),
                    Direction::Right    => right(coords),
                },
                Some(false) => return coords,
                None        => {
                    return coords
                }
            }
        }
    }
}

impl<T> WriteableGrid for Grid<T> where T: WriteableCell + Default {
    type Cell = T;

    fn writeable(&mut self, coords: Coords) -> Option<&mut Self::Cell> {
        self.fill_to(coords);
        self.get_mut(coords)
    }

    fn best_fit_for_region(&self, region: Region) -> Coords {
        let x_offset = self.max_width().map_or(0, |width| {
            region.right.saturating_sub(width as u32)
        });
        let y_offset = self.max_height().map_or(0, |height| {
            region.bottom.saturating_sub(height as u32)
        });
        Coords { x: region.left - x_offset, y: region.top - y_offset }
    }

    fn find_cell_to_extend(&self, coords: Coords) -> Option<Coords> {
        cell_to_extend(self, coords_before(coords, self.width as u32))
    }
}

impl<T> Resizeable for Grid<T> {
    fn resize_width(&mut self, width: u32) {
        let new_rem = (width as usize).saturating_sub(self.width);
        if let Some(ref mut rem) = self.rem_x {
            *rem = cmp::max(*rem, new_rem)
        }
    }

    fn resize_height(&mut self, height: u32) {
        let new_rem = (height as usize).saturating_sub(self.height);
        if let Some(ref mut rem) = self.rem_y {
            *rem = cmp::max(*rem, new_rem)
        }
    }
}

impl<T> Grid<T> {
    fn with_caps(max_x: usize, max_y: usize) -> Grid<T> {
        Grid::new(Some(max_x), Some(max_y))
    }

    fn with_infinite_scroll() -> Grid<T> {
        Grid::new(None, None)
    }

    fn new(max_x: Option<usize>, max_y: Option<usize>)
            -> Grid<T> {
        Grid {
            width: 0,
            height: 0,
            data: VecDeque::new(),
            rem_x: max_x,
            rem_y: max_y,
        }
    }

    fn max_width(&self) -> Option<usize> {
        self.rem_x.map(|x| x + self.width)
    }

    fn max_height(&self) -> Option<usize> {
        self.rem_y.map(|y| y + self.height)
    }

    fn bounds(&self) -> Option<Region> {
        if self.width > 0 && self.height > 0 {
            Some(Region::new(0, 0, self.width as u32, self.height as u32))
        } else { None }
    }

    fn get(&self, coords: Coords) -> Option<&T> {
        self.bounds().and_then(move |bounds| if bounds.contains(coords) { 
            Some(&self.data[linearize(self.width, coords)])
        } else { None })
    }

    fn get_mut(&mut self, coords: Coords) -> Option<&mut T> {
        self.bounds().and_then(move |bounds| if bounds.contains(coords) { 
            Some(&mut self.data[linearize(self.width, coords)])
        } else { None })
    }
}

impl<T: Default> Grid<T> {
    pub fn scroll(&mut self, direction: Direction, n: u32) {
        let n = n as usize;
        use datatypes::Direction::*;
        match direction {
            Up if self.rem_y != Some(0)     => self.extend_up(n),
            Up if n >= self.height          => self.data.clear(),
            Up                              => self.shift_up(n),
            Down if self.rem_y != Some(0)   => self.extend_down(n),
            Down if n >= self.height        => self.data.clear(),
            Down                            => self.shift_down(n),
            Left if self.rem_x != Some(0)   => self.extend_left(n),
            Left if n >= self.width         => self.data.clear(),
            Left                            => self.shift_left(n),
            Right if self.rem_x != Some(0)  => self.extend_right(n),
            Right if n >= self.width        => self.data.clear(),
            Right                           => self.shift_right(n),
        }
    }

    fn extend_up(&mut self, n: usize) {
        let rem_or_n = self.rem_y.map_or(n, |y| cmp::min(y, n));
        for _ in 0..(rem_or_n * self.width) {
            self.data.push_front(T::default());
        }
        self.height += rem_or_n;
        if self.rem_y.map_or(false, |y| n > y) {
            let rem = n - self.rem_y.unwrap();
            self.shift_up(rem);
        }
        self.rem_y = self.rem_y.map(|y| y.saturating_sub(n));
        if self.height > 0 && self.width == 0 {
            self.width = 1;
            self.rem_x.as_mut().map(|x| *x -= 1);
        }
    }

    fn extend_down(&mut self, n: usize) {
        let rem_or_n = self.rem_y.map_or(n, |y| cmp::min(y, n));
        for _ in 0..(rem_or_n * self.width) {
            self.data.push_back(T::default());
        }
        self.height += rem_or_n;
        if self.rem_y.map_or(false, |y| n > y) {
            let rem = n - self.rem_y.unwrap();
            self.shift_down(rem);
        }
        self.rem_y = self.rem_y.map(|y| y.saturating_sub(n));
        if self.height > 0 && self.width == 0 {
            self.width = 1;
            self.rem_x.as_mut().map(|x| *x -= 1);
        }
    }

    fn extend_left(&mut self, n: usize) {
        let rem_or_n = self.rem_x.map_or(n, |x| cmp::min(x, n));
        for i in 0..rem_or_n {
            for j in (1..self.height).rev() {
                self.data.insert((self.width + i) * j, T::default());
            }
            self.data.push_front(T::default());
        }
        self.width += rem_or_n;
        if self.rem_x.map_or(false, |x| n > x) {
            let rem = n - self.rem_x.unwrap();
            self.shift_left(rem);
        }
        self.rem_x = self.rem_x.map(|x| x.saturating_sub(n));
        if self.width > 0 && self.height == 0 {
            self.height = 1;
            self.rem_y.as_mut().map(|y| *y -= 1);
        }
    }

    fn extend_right(&mut self, n: usize) {
        let rem_or_n = self.rem_x.map_or(n, |x| cmp::min(x, n));
        for i in 0..rem_or_n {
            for j in (1..self.height).rev() {
                self.data.insert((self.width + i) * j, T::default());
            }
            self.data.push_back(T::default());
        }
        self.width += rem_or_n;
        if self.rem_x.map_or(false, |x| n > x) {
            let rem = n - self.rem_x.unwrap();
            self.shift_right(rem);
        }
        self.rem_x = self.rem_x.map(|x| x.saturating_sub(n));
        if self.width > 0 && self.height == 0 {
            self.height = 1;
            self.rem_y.as_mut().map(|y| *y -= 1);
        }
    }

    fn shift_up(&mut self, n: usize) {
        for _ in 0..(n * self.width) {
            self.data.pop_back();
            self.data.push_front(T::default());
        }
    }

    fn shift_down(&mut self, n: usize) {
        for _ in 0..(n * self.width) {
            self.data.pop_front();
            self.data.push_back(T::default());
        }
    }

    fn shift_left(&mut self, n: usize) {
        for _ in 0..n {
            self.data.pop_back();
            self.data.push_front(T::default());
            for i in 1..self.height {
                self.data[i * self.width] = T::default();
            }
        }
    }

    fn shift_right(&mut self, n: usize) {
        for _ in 0..n {
            self.data.pop_front();
            self.data.push_back(T::default());
            for i in 1..self.height {
                self.data[(i * self.width) - 1] = T::default();
            }
        }
    }

    fn fill_to(&mut self, Coords { x, y }: Coords) {
        if x as usize >= self.width { self.fill_to_width(x as usize + 1); }
        if y as usize >= self.height { self.fill_to_height(y as usize + 1); }
    }

    fn fill_to_width(&mut self, width: usize) {
        let extension = width.saturating_sub(self.width);
        self.extend_right(extension);
    }

    fn fill_to_height(&mut self, height: usize) {
        let extension = height.saturating_sub(self.height);
        self.extend_down(extension);
    }
}

fn linearize(width: usize, Coords { x, y }: Coords) -> usize {
    y as usize * width + x as usize
}

impl<'a, T> IntoIterator for &'a Grid<T> {
    type IntoIter = <&'a VecDeque<T> as IntoIterator>::IntoIter;
    type Item = <&'a VecDeque<T> as IntoIterator>::Item;
    fn into_iter(self) -> Self::IntoIter {
        (&self.data).into_iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Grid<T> {
    type IntoIter = <&'a mut VecDeque<T> as IntoIterator>::IntoIter;
    type Item = <&'a mut VecDeque<T> as IntoIterator>::Item;
    fn into_iter(self) -> Self::IntoIter {
        (&mut self.data).into_iter()
    }
}

pub fn coords_before(Coords { x, y }: Coords, width: u32) -> Coords {
    match (x == 0, y == 0) {
        (true, true)    => Coords { x: x, y: y },
        (true, _)       => Coords { x: width - 1, y: y - 1},
        (_, _)          => Coords { x: x - 1, y: y },
    }
}

pub fn cell_to_extend<T: WriteableCell>(grid: &Grid<T>, coords: Coords) -> Option<Coords> {
    if let Some(cell) = grid.get(coords) {
        if cell.is_extendable() {
            Some(coords)
        } else if let Some(coords) = cell.source() {
            cell_to_extend(grid, coords)
        } else { None }
    } else { None }
}

#[cfg(test)]
mod tests {

    use datatypes::{Coords, Region};
    use datatypes::Direction::*;
    use terminal::interfaces::Resizeable;

    use super::Grid;

    fn run_test<F: Fn(Grid<i32>, usize, usize)>(test: F, new_w: usize, new_h: usize) {
        fn fill(grid: &mut Grid<i32>) {
            grid.fill_to(Coords { x: 7, y: 7 });
            for i in grid { *i = 1; }
        }
        test({ let mut grid = Grid::with_caps(8, 8); fill(&mut grid); grid }, 8, 8);
        test({ let mut grid = Grid::with_caps(10, 8); fill(&mut grid); grid }, new_w, 8);
        test({ let mut grid = Grid::with_caps(8, 10); fill(&mut grid); grid }, 8, new_h);
        test({ let mut grid = Grid::with_caps(10, 10); fill(&mut grid); grid },
             new_w, new_h);
    }

    #[test]
    fn scroll_left() {
        run_test(|mut grid, width, height| {
            grid.scroll(Left, 3);
            for i in 0..grid.height {
                assert_eq!(*grid.get(Coords {x:0, y:i as u32}).unwrap(), 0);
                assert_eq!(*grid.get(Coords {x:1, y:i as u32}).unwrap(), 0);
                assert_eq!(*grid.get(Coords {x:2, y:i as u32}).unwrap(), 0);
                assert_eq!(*grid.get(Coords {x:3, y:i as u32}).unwrap(), 1);
            }
            assert_eq!(width, grid.width);
            assert_eq!(height, grid.height);
            assert_eq!(grid.data.len(), width * height);
        }, 10, 8);
    }

    #[test]
    fn scroll_right() {
        run_test(|mut grid, width, height| {
            grid.scroll(Right, 3);
            for i in 0..grid.height {
                assert_eq!(*grid.get(Coords {x:width as u32-1, y:i as u32}).unwrap(), 0);
                assert_eq!(*grid.get(Coords {x:width as u32-2, y:i as u32}).unwrap(), 0);
                assert_eq!(*grid.get(Coords {x:width as u32-3, y:i as u32}).unwrap(), 0);
                assert_eq!(*grid.get(Coords {x:width as u32-4, y:i as u32}).unwrap(), 1);
            }
            assert_eq!(width, grid.width);
            assert_eq!(height, grid.height);
            assert_eq!(grid.data.len(), width * height);
        }, 10, 8);
    }

    #[test]
    fn scroll_up() {
        run_test(|mut grid, width, height| {
            grid.scroll(Up, 3);
            for i in 0..grid.width {
                assert_eq!(*grid.get(Coords {x:i as u32, y:0}).unwrap(), 0);
                assert_eq!(*grid.get(Coords {x:i as u32, y:1}).unwrap(), 0);
                assert_eq!(*grid.get(Coords {x:i as u32, y:2}).unwrap(), 0);
                assert_eq!(*grid.get(Coords {x:i as u32, y:3}).unwrap(), 1);
            }
            assert_eq!(width, grid.width);
            assert_eq!(height, grid.height);
            assert_eq!(grid.data.len(), width * height);
        }, 8, 10);
    }

    #[test]
    fn scroll_down() {
        run_test(|mut grid, width, height| {
            grid.scroll(Down, 3);
            for i in 0..grid.width {
                assert_eq!(*grid.get(Coords {x:i as u32, y:height as u32-1}).unwrap(), 0);
                assert_eq!(*grid.get(Coords {x:i as u32, y:height as u32-2}).unwrap(), 0);
                assert_eq!(*grid.get(Coords {x:i as u32, y:height as u32-3}).unwrap(), 0);
                assert_eq!(*grid.get(Coords {x:i as u32, y:height as u32-4}).unwrap(), 1);
            }
            assert_eq!(width, grid.width);
            assert_eq!(height, grid.height);
            assert_eq!(grid.data.len(), width * height);
        }, 8, 10);
    }

    #[test]
    fn max_width_and_height() {
        run_test(|grid, width, height| {
            assert_eq!(grid.max_width(), Some(width));
            assert_eq!(grid.max_height(), Some(height));
        }, 10, 10);
    }

    #[test]
    fn bounds() {
        run_test(|grid, _, _| assert_eq!(grid.bounds(), Some(Region::new(0, 0, 8, 8))), 8, 8);
    }

    #[test]
    fn resize_width() {
        run_test(|mut grid, width, _| {
            grid.resize_width(12);
            assert_eq!(grid.max_width(), Some(12));
        }, 12, 8)
    }

    #[test]
    fn resize_height() {
        run_test(|mut grid, height, _| {
            grid.resize_height(12);
            assert_eq!(grid.max_height(), Some(12));
        }, 8, 12)
    }
}
