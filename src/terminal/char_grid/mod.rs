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
use std::collections::HashMap;
use std::ops::Index;

use unicode_width::*;

use cfg::CONFIG;
use datatypes::{Area, CellData, Coords, CoordsIter, Direction, Movement, Region, Style, move_within};
use datatypes::Area::*;
use datatypes::Movement::*;
use datatypes::Direction::*;

mod cell;
mod cursor;
mod grid;
mod styles;
mod tooltip;

pub use self::cell::{CharCell, ImageData};
pub use self::cursor::Cursor;
pub use self::grid::Grid;
pub use self::styles::Styles;
pub use self::tooltip::Tooltip;

pub struct CharGrid {
    grid: Grid<CharCell>,
    cursor: Cursor,
    tooltips: HashMap<Coords, Tooltip>,
    window: Region,
}

impl CharGrid {
    pub fn new(width: u32, height: u32, retain_offscreen_state: bool) -> CharGrid {
        let grid = if retain_offscreen_state {
            Grid::with_x_y_caps(width as usize, height as usize,
                                CONFIG.scrollback as usize, CONFIG.scrollback as usize)
        } else {
            Grid::new(width as usize, height as usize)
        };
        CharGrid {
            grid: grid,
            cursor: Cursor::default(),
            tooltips: HashMap::new(),
            window: Region::new(0, 0, width, height),
        }
    }

    pub fn resize_window(&mut self, region: Region) {
        if self.grid_width() < region.width() {
            let n = (region.width() - self.grid_width()) * self.grid_height();
            self.grid.add_to_right(vec![CharCell::default(); n as usize]);
        }
        if self.grid_height() < region.height() {
            let n = (region.height() - self.grid_height()) * self.grid_width();
            self.grid.add_to_bottom(vec![CharCell::default(); n as usize]);
        }
        self.window = Region {
            right: self.window.left + region.width(),
            bottom: self.window.top + region.height(),
            ..self.window
        };
    }

    pub fn write(&mut self, data: CellData) {
        match data {
            CellData::Char(c)       => {
                let width = c.width().unwrap() as u32;
                self.grid[self.cursor.coords] = CharCell::character(c, self.cursor.text_style);
                let bounds = self.grid.bounds();
                let mut coords = self.cursor.coords;
                for _ in 1..width {
                    let next_coords = move_within(coords, To(Right, 1, false), bounds,
                                                  CONFIG.tab_stop);
                    if next_coords == coords { break; } else { coords = next_coords; }
                    self.grid[coords] = CharCell::Extension(self.cursor.coords,
                                                            self.cursor.text_style);
                }
                self.cursor.navigate(&mut self.grid, To(Right, 1, true));
            }
            CellData::ExtensionChar(c)  => {
                self.cursor.navigate(&mut self.grid, To(Left, 1, true));
                if !self.grid[self.cursor.coords].extend_by(c) {
                    self.cursor.navigate(&mut self.grid, To(Right, 1, true));
                    self.grid[self.cursor.coords] = CharCell::character(c, self.cursor.text_style);
                    self.cursor.navigate(&mut self.grid, To(Right, 1, true));
                }
            }
            CellData::Image { pos, width, height, data, mime }   => {
                let mut end = self.cursor.coords;
                end = move_within(end, To(Right, width, false), self.grid.bounds(), CONFIG.tab_stop);
                end = move_within(end, To(Down, height, false), self.grid.bounds(), CONFIG.tab_stop);
                let mut iter = CoordsIter::from_area(CursorBound(end),
                                                     self.cursor.coords, self.grid.bounds(),
                                                     CONFIG.tab_stop);
                if let Some(cu_coords) = iter.next() {
                    self.grid[cu_coords] = CharCell::image(data, self.cursor.coords, mime, pos,
                                                           width, height, self.cursor.text_style);
                    for coords in iter {
                        self.grid[coords] = CharCell::Extension(cu_coords, self.cursor.text_style);
                    }
                    self.cursor.navigate(&mut self.grid, To(Right, 1, true));
                }
            }
        }
    }

    pub fn move_cursor(&mut self, movement: Movement) {
        self.cursor.navigate(&mut self.grid, movement);
        self.window = self.window.move_to_contain(self.cursor.coords);
    }

    pub fn add_tooltip(&mut self, coords: Coords, tooltip: String) {
        self.tooltips.insert(coords, Tooltip::Basic(tooltip));
    }

    pub fn remove_tooltip(&mut self, coords: Coords) {
        self.tooltips.remove(&coords);
    }

    pub fn add_drop_down(&mut self, coords: Coords, options: Vec<String>) {
        self.tooltips.insert(coords, Tooltip::Menu { options: options, position: None });
    }

    pub fn scroll(&mut self, dir: Direction, n: u32) {
        self.grid.scroll(n as usize, dir)
    }

    pub fn erase(&mut self, area: Area) {
        self.in_area(area, |grid, coords| grid[coords] = CharCell::default());
    }

    pub fn insert_blank_at(&mut self, n: u32) {
        let mut iter = CoordsIter::from_area(CursorTo(ToEdge(Right)),
                                             self.cursor.coords,
                                             self.grid.bounds(),
                                             CONFIG.tab_stop);
        iter.next();
        for coords in iter.rev().skip(n as usize) {
            self.grid.moveover(coords, Coords {x: coords.x + n, y: coords.y});
        }
    }

    pub fn remove_at(&mut self, n: u32) {
        self.in_area(CursorTo(ToEdge(Right)), |grid, coords| {
            if coords.x + n < grid.width as u32 {
                grid.moveover(Coords {x: coords.x + n, y: coords.y}, coords);
            }
        })
    }

    pub fn insert_rows_at(&mut self, n: u32, include: bool) {
        let region = if include {
            Region::new(0, self.cursor.coords.y, self.grid.width as u32, self.grid.height as u32)
        } else if self.cursor.coords.y + 1 == self.grid.width as u32 {
            return
        } else {
            Region::new(0, self.cursor.coords.y + 1, self.grid.width as u32, self.grid.height as u32)
        };
        for coords in CoordsIter::from_region(region).rev().skip(n as usize * self.grid.width) {
            self.grid.moveover(coords, Coords {x: coords.x, y: coords.y + n});
        }
    }

    pub fn remove_rows_at(&mut self, n: u32, include: bool) {
        self.in_area(BelowCursor(include), |grid, coords| {
            if coords.y + n < grid.height as u32 {
                grid.moveover(Coords {x: coords.x, y: coords.y + n}, coords);
            }
        })
    }

    pub fn set_style(&mut self, style: Style) {
        self.cursor.text_style.update(style);
    }

    pub fn reset_styles(&mut self) {
        self.cursor.text_style = Styles::default();
    }

    pub fn set_cursor_style(&mut self, style: Style) {
        self.cursor.style.update(style);
    }

    pub fn reset_cursor_styles(&mut self) {
        self.cursor.style = Styles::default();
    }

    pub fn set_style_in_area(&mut self, area: Area, style: Style) {
        self.in_area(area, |grid, coords| grid[coords].style_mut().update(style));
    }

    pub fn reset_styles_in_area(&mut self, area: Area) {
        self.in_area(area, |grid, coords| *grid[coords].style_mut() = Styles::default());
    }

    pub fn cursor_position(&self) -> Coords {
        self.cursor.coords
    }

    pub fn cursor_styles(&self) -> Styles {
        self.cursor.style
    }

    pub fn grid_width(&self) -> u32 {
        self.grid.width as u32
    }

    pub fn grid_height(&self) -> u32 {
        self.grid.height as u32
    }

    pub fn tooltip_at(&self, coords: Coords) -> Option<&Tooltip> {
        self.tooltips.get(&coords)
    }

    pub fn tooltip_at_mut(&mut self, coords: Coords) -> Option<&mut Tooltip> {
        self.tooltips.get_mut(&coords)
    }

    fn in_area<F>(&mut self, area: Area, f: F) where F: Fn(&mut Grid<CharCell>, Coords) {
        for coords in CoordsIter::from_area(area, self.cursor.coords, self.grid.bounds(),
                                            CONFIG.tab_stop) {
            f(&mut self.grid, coords);
        }
    }

}

impl<'a> IntoIterator for &'a CharGrid {
    type IntoIter = <&'a Grid<CharCell> as IntoIterator>::IntoIter;
    type Item = &'a CharCell;
    fn into_iter(self) -> Self::IntoIter {
        self.grid.into_iter()
    }
}

impl Index<Coords> for CharGrid {
    type Output = CharCell;
    fn index(&self, Coords {x, y}: Coords) -> &CharCell {
        let coords = Coords { x: x + self.window.left, y: y + self.window.top };
        assert!(self.window.contains(coords));
        &self.grid[coords]
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use cfg::CONFIG;
    use datatypes::{CellData, Coords, Direction, Movement, Region};

    fn run_test<F: Fn(CharGrid, u32)>(test: F) {
        test(CharGrid::new(10, 10, false), 10);
        test(CharGrid::new(10, 10, true), 11);
    }

    #[test]
    fn window_scrolls_with_cursor() {
        run_test(|mut grid, h| {
            grid.move_cursor(Movement::NextLine(10));
            assert_eq!(grid.window, Region::new(0, h - 10, 10, h));
        })
    }

    #[test]
    fn write() {
        run_test(|mut grid, _| {
            for c in vec![
                CellData::Char('Q'),
                CellData::Char('E'),
                CellData::ExtensionChar('\u{301}'),
            ].into_iter() { grid.write(c); }
            assert_eq!(grid.grid[Coords {x:0, y:0}].repr(), "Q");
            assert_eq!(grid.grid[Coords {x:1, y:0}].repr(), "E\u{301}");
        });
    }

    fn setup(grid: &mut CharGrid) {
        let mut chars = vec![
            CellData::Char('A'),
            CellData::Char('B'),
            CellData::Char('C'),
            CellData::Char('D'),
            CellData::Char('E'),
            CellData::Char('1'),
            CellData::Char('2'),
            CellData::Char('3'),
            CellData::Char('4'),
            CellData::Char('5'),
            CellData::Char('!'),
            CellData::Char('@'),
            CellData::Char('#'),
            CellData::Char('$'),
            CellData::Char('%'),
        ].into_iter();
        for _ in 0..3 {
            for c in chars.by_ref().take(5) { grid.write(c); }
            grid.move_cursor(Movement::NextLine(1));
        }
        grid.move_cursor(Movement::ToBeginning);
    }

    #[test]
    fn move_cursor() {
        run_test(|mut grid, h| {
            let movements = vec![
                (Movement::ToEdge(Direction::Down), Coords {x:0, y:9}),
                (Movement::Tab(Direction::Right, 1, false), Coords{x:CONFIG.tab_stop, y:9}),
                (Movement::NextLine(1), Coords{x:0, y:h-1}),
            ];
            for (mov, coords) in movements {
                grid.move_cursor(mov);
                assert_eq!(grid.cursor_position(), coords);
            }
            assert_eq!(grid.grid.height as u32, h);
        })
    }

    #[test]
    fn insert_blank_at() {
        run_test(|mut grid, _| {
            setup(&mut grid);
            grid.insert_blank_at(1);
            assert_eq!(grid.grid[Coords {x:0, y:0}].repr(), "A");
            assert_eq!(grid.grid[Coords {x:1, y:0}].repr(), "");
            assert_eq!(grid.grid[Coords {x:2, y:0}].repr(), "B");
            assert_eq!(grid.grid[Coords {x:3, y:0}].repr(), "C");
            assert_eq!(grid.grid[Coords {x:4, y:0}].repr(), "D");
            assert_eq!(grid.grid[Coords {x:5, y:0}].repr(), "E");
            grid.move_cursor(Movement::NextLine(1));
            grid.insert_blank_at(2);
            assert_eq!(grid.grid[Coords {x:0, y:1}].repr(), "1");
            assert_eq!(grid.grid[Coords {x:1, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:2, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:3, y:1}].repr(), "2");
            assert_eq!(grid.grid[Coords {x:4, y:1}].repr(), "3");
            assert_eq!(grid.grid[Coords {x:5, y:1}].repr(), "4");
            assert_eq!(grid.grid[Coords {x:6, y:1}].repr(), "5");
            grid.move_cursor(Movement::NextLine(1));
            grid.insert_blank_at(3);
            assert_eq!(grid.grid[Coords {x:0, y:2}].repr(), "!");
            assert_eq!(grid.grid[Coords {x:1, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:2, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:3, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:4, y:2}].repr(), "@");
            assert_eq!(grid.grid[Coords {x:5, y:2}].repr(), "#");
            assert_eq!(grid.grid[Coords {x:6, y:2}].repr(), "$");
            assert_eq!(grid.grid[Coords {x:7, y:2}].repr(), "%");
        })
    }

    #[test]
    fn remove_at() {
        run_test(|mut grid, _| {
            setup(&mut grid);
            grid.remove_at(1);
            assert_eq!(grid.grid[Coords {x:0, y:0}].repr(), "B");
            assert_eq!(grid.grid[Coords {x:1, y:0}].repr(), "C");
            assert_eq!(grid.grid[Coords {x:2, y:0}].repr(), "D");
            assert_eq!(grid.grid[Coords {x:3, y:0}].repr(), "E");
            assert_eq!(grid.grid[Coords {x:4, y:0}].repr(), "");
            grid.move_cursor(Movement::NextLine(1));
            grid.remove_at(2);
            assert_eq!(grid.grid[Coords {x:0, y:1}].repr(), "3");
            assert_eq!(grid.grid[Coords {x:1, y:1}].repr(), "4");
            assert_eq!(grid.grid[Coords {x:2, y:1}].repr(), "5");
            assert_eq!(grid.grid[Coords {x:3, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:4, y:1}].repr(), "");
            grid.move_cursor(Movement::NextLine(1));
            grid.remove_at(3);
            assert_eq!(grid.grid[Coords {x:0, y:2}].repr(), "$");
            assert_eq!(grid.grid[Coords {x:1, y:2}].repr(), "%");
            assert_eq!(grid.grid[Coords {x:2, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:3, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:4, y:2}].repr(), "");
        })
    }

    #[test]
    fn insert_rows_at() {
        run_test(|mut grid, _| {
            setup(&mut grid);
            grid.insert_rows_at(2, false);
            assert_eq!(grid.grid[Coords {x:0, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:1, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:2, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:3, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:4, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:0, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:1, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:2, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:3, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:4, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:0, y:3}].repr(), "1");
            assert_eq!(grid.grid[Coords {x:1, y:3}].repr(), "2");
            assert_eq!(grid.grid[Coords {x:2, y:3}].repr(), "3");
            assert_eq!(grid.grid[Coords {x:3, y:3}].repr(), "4");
            assert_eq!(grid.grid[Coords {x:4, y:3}].repr(), "5");
            grid.insert_rows_at(3, true);
            assert_eq!(grid.grid[Coords {x:0, y:0}].repr(), "");
            assert_eq!(grid.grid[Coords {x:1, y:0}].repr(), "");
            assert_eq!(grid.grid[Coords {x:2, y:0}].repr(), "");
            assert_eq!(grid.grid[Coords {x:3, y:0}].repr(), "");
            assert_eq!(grid.grid[Coords {x:4, y:0}].repr(), "");
            assert_eq!(grid.grid[Coords {x:0, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:1, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:2, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:3, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:4, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:0, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:1, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:2, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:3, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:4, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:0, y:3}].repr(), "A");
            assert_eq!(grid.grid[Coords {x:1, y:3}].repr(), "B");
            assert_eq!(grid.grid[Coords {x:2, y:3}].repr(), "C");
            assert_eq!(grid.grid[Coords {x:3, y:3}].repr(), "D");
            assert_eq!(grid.grid[Coords {x:4, y:3}].repr(), "E");
        })
    }

    #[test]
    fn remove_rows_at() {
        run_test(|mut grid, _| {
            setup(&mut grid);
            grid.remove_rows_at(2, true);
            assert_eq!(grid.grid[Coords {x:0, y:0}].repr(), "!");
            assert_eq!(grid.grid[Coords {x:1, y:0}].repr(), "@");
            assert_eq!(grid.grid[Coords {x:2, y:0}].repr(), "#");
            assert_eq!(grid.grid[Coords {x:3, y:0}].repr(), "$");
            assert_eq!(grid.grid[Coords {x:4, y:0}].repr(), "%");
        })
    }

}
