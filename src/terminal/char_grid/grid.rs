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
use std::iter;
use std::mem;
use std::ops::{Index, IndexMut};

use datatypes::{Coords, Direction, Region};

pub struct Grid<T> {
    pub width: usize,
    pub height: usize,
    pub scrolls_x: bool,
    pub scrolls_y: bool,
    data: VecDeque<T>,
    rem_x: usize,
    rem_y: usize,
    default: T,
}

impl<T: Clone> Grid<T> {

    pub fn new(width: usize, height: usize, default: T) -> Grid<T> {
        Grid::with_x_y_caps(width, height, 0, 0, default)
    }

    pub fn with_x_cap(width: usize, height: usize, max_x: usize, default: T) -> Grid<T> {
        Grid::with_x_y_caps(width, height, max_x, 0, default)
    }

    pub fn with_y_cap(width: usize, height: usize, max_y: usize, default: T) -> Grid<T> {
        Grid::with_x_y_caps(width, height, 0, max_y, default)
    }

    pub fn with_x_y_caps(w: usize, h: usize, max_x: usize, max_y: usize, default: T) -> Grid<T> {
        Grid {
            width: w,
            height: h,
            scrolls_x: max_x != 0,
            scrolls_y: max_y != 0,
            data: iter::repeat(default.clone()).take(w * h).collect(),
            rem_x: max_x.saturating_sub(w),
            rem_y: max_y.saturating_sub(h),
            default: default,
        }
    }

    pub fn bounds(&self) -> Region {
        Region::new(0, 0, self.width as u32, self.height as u32)
    }

    pub fn add_to_top(&mut self, data: Vec<T>) {
        assert!(data.len() % self.width == 0);
        self.height += data.len() / self.width;
        for item in data {
            self.data.push_front(item);
        }
    }

    pub fn add_to_bottom(&mut self, data: Vec<T>) {
        assert!(data.len() % self.width == 0);
        self.height += data.len() / self.width;
        for item in data {
            self.data.push_back(item);
        }
    }

    pub fn remove_from_top(&mut self, n: usize) -> Vec<T> {
        assert!(n < self.height);
        self.height -= n;
        let n = n * self.width;
        self.data.drain(..n).collect()
    }

    pub fn remove_from_bottom(&mut self, n: usize) -> Vec<T> {
        assert!(n < self.height);
        self.height -= n;
        let n = self.data.len() - (n * self.width);
        self.data.drain(n..).collect()
    }

    pub fn add_to_left(&mut self, data: Vec<T>) {
        assert!(data.len() % self.height == 0);
        let extra_width = data.len() / self.height;
        let width = self.width;
        self.width += extra_width;
        let iter = data.into_iter().enumerate().map(|(idx, item)| {
            ((idx / extra_width) * width, item)
        }).rev();
        for (idx, item) in iter {
            self.data.insert(idx, item);
        }
    }

    pub fn remove_from_left(&mut self, n: usize) -> Vec<T> {
        assert!(n < self.width);
        let width = self.width;
        let len = self.data.len();
        self.width -= n;
        (0..len).filter(|&x| (x % width) < n)
                .rev().map(|idx| self.data.remove(idx).unwrap())
                .collect()
    }

    pub fn add_to_right(&mut self, data: Vec<T>) {
        assert!(data.len() % self.height == 0);
        let extra_width = data.len() / self.height;
        let width = self.width;
        self.width += extra_width;
        let iter = data.into_iter().enumerate().map(|(idx, item)| {
            ((idx / extra_width) * width + width, item)
        }).rev();
        for (idx, item) in iter {
            self.data.insert(idx, item);
        }
    }

    pub fn remove_from_right(&mut self, n: usize) -> Vec<T> {
        assert!(n < self.width);
        let width = self.width;
        let len = self.data.len();
        self.width -= n;
        (0..len).filter(|&x| (x % width) >= width - n)
                .rev().map(|idx| self.data.remove(idx).unwrap())
                .collect()
    }

    pub fn scroll(&mut self, n: usize, direction: Direction) {
        use datatypes::Direction::*;
        match direction {
            Up if self.rem_y != 0           => self.extend_up(n),
            Up if n >= self.height          => self.data.clear(),
            Up                              => self.shift_up(n),
            Down if self.rem_y != 0         => self.extend_down(n),
            Down if n >= self.height        => self.data.clear(),
            Down                            => self.shift_down(n),
            Left if self.rem_x != 0         => self.extend_left(n),
            Left if n >= self.width         => self.data.clear(),
            Left                            => self.shift_left(n),
            Right if self.rem_x != 0        => self.extend_right(n),
            Right if n >= self.width        => self.data.clear(),
            Right                           => self.shift_right(n),
        }
    }

    pub fn moveover(&mut self, from: Coords, to: Coords) {
        let default = self.default.clone();
        self[to] = mem::replace(&mut self[from], default);
    }

    fn extend_up(&mut self, n: usize) {
        let rem_or_n = cmp::min(self.rem_y, n);
        for _ in 0..(rem_or_n * self.width) {
            self.data.push_front(self.default.clone());
        }
        self.height += rem_or_n;
        if n > self.rem_y {
            let rem = n - self.rem_y;
            self.shift_up(rem);
        }
        self.rem_y = self.rem_y.saturating_sub(n);
    }

    fn extend_down(&mut self, n: usize) {
        let rem_or_n = cmp::min(self.rem_y, n);
        for _ in 0..(rem_or_n * self.width) {
            self.data.push_back(self.default.clone());
        }
        self.height += rem_or_n;
        if n > self.rem_y {
            let rem = n - self.rem_y;
            self.shift_down(rem);
        }
        self.rem_y = self.rem_y.saturating_sub(n);
    }

    fn extend_left(&mut self, n: usize) {
        let rem_or_n = cmp::min(self.rem_x, n);
        for i in 0..rem_or_n {
            for j in (1..self.height).rev() {
                self.data.insert((self.width + i) * j, self.default.clone());
            }
            self.data.push_front(self.default.clone());
        }
        self.width += rem_or_n;
        if n > self.rem_x {
            let rem = n - self.rem_x;
            self.shift_left(rem);
        }
        self.rem_y = self.rem_y.saturating_sub(n);
    }

    fn extend_right(&mut self, n: usize) {
        let rem_or_n = cmp::min(self.rem_x, n);
        for i in 0..rem_or_n {
            for j in (1..self.height).rev() {
                self.data.insert((self.width + i) * j, self.default.clone());
            }
            self.data.push_back(self.default.clone());
        }
        self.width += rem_or_n;
        if n > self.rem_x {
            let rem = n - self.rem_x;
            self.shift_right(rem);
        }
        self.rem_y = self.rem_y.saturating_sub(n);
    }

    fn shift_up(&mut self, n: usize) {
        for _ in 0..(n * self.width) {
            self.data.pop_back();
            self.data.push_front(self.default.clone());
        }
    }

    fn shift_down(&mut self, n: usize) {
        for _ in 0..(n * self.width) {
            self.data.pop_front();
            self.data.push_back(self.default.clone());
        }
    }

    fn shift_left(&mut self, n: usize) {
        for _ in 0..n {
            self.data.pop_back();
            self.data.push_front(self.default.clone());
            for i in 1..self.height {
                self.data[i * self.width] = self.default.clone();
            }
        }
    }

    fn shift_right(&mut self, n: usize) {
        for _ in 0..n {
            self.data.pop_front();
            self.data.push_back(self.default.clone());
            for i in 1..self.height {
                self.data[(i * self.width) - 1] = self.default.clone();
            }
        }
    }

}

impl<T> Index<Coords> for Grid<T> {
    type Output = T;
    fn index(&self, idx: Coords) -> &T {
        assert!(self.width > idx.x as usize, "{} index outside of x bounds", idx.x);
        assert!(self.height > idx.y as usize, "{} index outside of y bounds", idx.y);
        &self.data[(idx.y as usize * self.width) + idx.x as usize]
    }
}

impl<T> IndexMut<Coords> for Grid<T> {
    fn index_mut(&mut self, idx: Coords) -> &mut T {
        assert!(self.width > idx.x as usize, "{} index outside of x bounds", idx.x);
        assert!(self.height > idx.y as usize, "{} index otuside of y bounds", idx.y);
        &mut self.data[(idx.y as usize * self.width) + idx.x as usize]
    }
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

#[cfg(test)]
mod tests {

    use datatypes::Coords;
    use datatypes::Direction::*;

    use super::Grid;

    fn run_test<F: Fn(Grid<i32>, usize, usize)>(test: F, new_w: usize, new_h: usize) {
        let fill = |grid: &mut Grid<i32>| for i in grid { *i = 1; };
        test({ let mut grid = Grid::new(8, 8, 0); fill(&mut grid); grid }, 8, 8);
        test({ let mut grid = Grid::with_x_cap(8, 8, 10, 0); fill(&mut grid); grid }, new_w, 8);
        test({ let mut grid = Grid::with_y_cap(8, 8, 10, 0); fill(&mut grid); grid }, 8, new_h);
        test({ let mut grid = Grid::with_x_y_caps(8, 8, 10, 10, 0); fill(&mut grid); grid },
             new_w, new_h);
    }

    #[test]
    fn add_to_top() {
        run_test(|mut grid, width, _| {
            grid.add_to_top(vec![0; 8]);
            for i in 0..grid.width {
                assert_eq!(grid[Coords {x:i as u32, y:0}], 0);
                assert_eq!(grid[Coords {x:i as u32, y:1}], 1);
            }
            assert_eq!(width, grid.width);
            assert_eq!(9, grid.height);
            assert_eq!(grid.data.len(), width * 9);
        }, 8, 9)
    }

    #[test]
    fn add_to_bottom() {
        run_test(|mut grid, width, _| {
            grid.add_to_bottom(vec![0; 8]);
            for i in 0..grid.width {
                assert_eq!(grid[Coords {x:i as u32, y:8}], 0);
                assert_eq!(grid[Coords {x:i as u32, y:7}], 1);
            }
            assert_eq!(width, grid.width);
            assert_eq!(9, grid.height);
            assert_eq!(grid.data.len(), width * 9);
        }, 8, 9);
    }

    #[test]
    fn add_to_left() {
        run_test(|mut grid, _, height| {
            grid.add_to_left(vec![0; 8]);
            for i in 0..grid.height {
                assert_eq!(grid[Coords {x:0, y:i as u32}], 0);
                assert_eq!(grid[Coords {x:1, y:i as u32}], 1);
            }
            assert_eq!(9, grid.width);
            assert_eq!(height, grid.height);
            assert_eq!(grid.data.len(), height * 9);
        }, 9, 8)
    }

    #[test]
    fn add_to_right() {
        run_test(|mut grid, _, height| {
            grid.add_to_right(vec![0; 8]);
            for i in 0..grid.height {
                assert_eq!(grid[Coords {x:8, y:i as u32}], 0);
                assert_eq!(grid[Coords {x:7, y:i as u32}], 1);
            }
            assert_eq!(9, grid.width);
            assert_eq!(height, grid.height);
            assert_eq!(grid.data.len(), height * 9);
        }, 9, 8);
    }

    #[test]
    fn remove_from_top() {
        run_test(|mut grid, width, _| {
            assert_eq!(grid.remove_from_top(2), vec![1; 16]);
            assert_eq!(width, grid.width);
            assert_eq!(6, grid.height);
            assert_eq!(grid.data.len(), width * 6);
        }, 8, 6);
    }

    #[test]
    fn remove_from_bottom() {
        run_test(|mut grid, width, _| {
            assert_eq!(grid.remove_from_bottom(2), vec![1; 16]);
            assert_eq!(width, grid.width);
            assert_eq!(6, grid.height);
            assert_eq!(grid.data.len(), width * 6);
        }, 8, 6);
    }

    #[test]
    fn remove_from_left() {
        run_test(|mut grid, _, height| {
            assert_eq!(grid.remove_from_left(2), vec![1; 16]);
            assert_eq!(6, grid.width);
            assert_eq!(height, grid.height);
            assert_eq!(grid.data.len(), height * 6);
        }, 6, 8)
    }

    #[test]
    fn remove_from_right() {
        run_test(|mut grid, _, height| {
            assert_eq!(grid.remove_from_right(2), vec![1; 16]);
            assert_eq!(6, grid.width);
            assert_eq!(height, grid.height);
            assert_eq!(grid.data.len(), height * 6);
        }, 6, 8)
    }

    #[test]
    fn scroll_left() {
        run_test(|mut grid, width, height| {
            grid.scroll(3, Left);
            for i in 0..grid.height {
                assert_eq!(grid[Coords {x:0, y:i as u32}], 0);
                assert_eq!(grid[Coords {x:1, y:i as u32}], 0);
                assert_eq!(grid[Coords {x:2, y:i as u32}], 0);
                assert_eq!(grid[Coords {x:3, y:i as u32}], 1);
            }
            assert_eq!(width, grid.width);
            assert_eq!(height, grid.height);
            assert_eq!(grid.data.len(), width * height);
        }, 10, 8);
    }

    #[test]
    fn scroll_right() {
        run_test(|mut grid, width, height| {
            grid.scroll(3, Right);
            for i in 0..grid.height {
                assert_eq!(grid[Coords {x:width as u32-1, y:i as u32}], 0);
                assert_eq!(grid[Coords {x:width as u32-2, y:i as u32}], 0);
                assert_eq!(grid[Coords {x:width as u32-3, y:i as u32}], 0);
                assert_eq!(grid[Coords {x:width as u32-4, y:i as u32}], 1);
            }
            assert_eq!(width, grid.width);
            assert_eq!(height, grid.height);
            assert_eq!(grid.data.len(), width * height);
        }, 10, 8);
    }

    #[test]
    fn scroll_up() {
        run_test(|mut grid, width, height| {
            grid.scroll(3, Up);
            for i in 0..grid.width {
                assert_eq!(grid[Coords {x:i as u32, y:0}], 0);
                assert_eq!(grid[Coords {x:i as u32, y:1}], 0);
                assert_eq!(grid[Coords {x:i as u32, y:2}], 0);
                assert_eq!(grid[Coords {x:i as u32, y:3}], 1);
            }
            assert_eq!(width, grid.width);
            assert_eq!(height, grid.height);
            assert_eq!(grid.data.len(), width * height);
        }, 8, 10);
    }

    #[test]
    fn scroll_down() {
        run_test(|mut grid, width, height| {
            grid.scroll(3, Down);
            for i in 0..grid.width {
                assert_eq!(grid[Coords {x:i as u32, y:height as u32-1}], 0);
                assert_eq!(grid[Coords {x:i as u32, y:height as u32-2}], 0);
                assert_eq!(grid[Coords {x:i as u32, y:height as u32-3}], 0);
                assert_eq!(grid[Coords {x:i as u32, y:height as u32-4}], 1);
            }
            assert_eq!(width, grid.width);
            assert_eq!(height, grid.height);
            assert_eq!(grid.data.len(), width * height);
        }, 8, 10);
    }

}
