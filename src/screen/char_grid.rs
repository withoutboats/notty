use std::cmp::{self, Ordering};

use unicode_width::*;

use cfg;
use datatypes::{Area, CellData, Coords, Movement, Region, Style, Vector};
use datatypes::Area::*;
use datatypes::Movement::*;
use datatypes::Direction::*;

use screen::{CharCell, Cursor, Grid, Styles};

pub struct CharGrid {
    grid: Grid<CharCell>,
    cursor: Cursor,
    pub grid_width: u32,
    pub grid_height: u32,
}

impl CharGrid {
    pub fn new(w: u32, h: u32, scroll_x: bool, scroll_y: bool) -> CharGrid {
        let grid = match (scroll_x, scroll_y) {
            (false, false)  => Grid::new(w as usize, h as usize),
            (false, true)   => Grid::with_y_cap(w as usize, h as usize, cfg::SCROLLBACK as usize),
            (true, false)   => unimplemented!(),
            (true, true)    => unimplemented!(),
        };
        CharGrid {
            grid: grid,
            cursor: Cursor::default(),
            grid_width: w,
            grid_height: h,
        }
    }

    pub fn set_height(&mut self, h: u32) {
        if self.grid.scrolls_y { return; }
        match self.grid_height.cmp(&h) {
            Ordering::Greater   => {
                let n = (self.grid_height - h) as usize;
                self.grid.remove_from_bottom(n);
            }
            Ordering::Equal     => (),
            Ordering::Less      => {
                let n = ((h - self.grid_height) * self.grid_width) as usize;
                self.grid.add_to_bottom(vec![CharCell::default(); n]);
            }
        }
        self.grid_height = h;
    }

    pub fn set_width(&mut self, w: u32) {
        if self.grid.scrolls_x { return; }
        match self.grid_width.cmp(&w) {
            Ordering::Greater   => {
                let n = (self.grid_width - w) as usize;
                self.grid.remove_from_right(n);
            }
            Ordering::Equal     => (),
            Ordering::Less      => {
                let n = ((w - self.grid_width) * self.grid_height) as usize;
                self.grid.add_to_right(vec![CharCell::default(); n]);
            }
        }
        self.grid_width = w;
    }

    pub fn write(&mut self, data: CellData) {
        match data {
            CellData::Char(c)       => {
                let width = c.width().unwrap() as u32;
                self.grid[self.cursor.coords] = CharCell::character(c, self.cursor.text_style);
                for i in 1..width {
                    //bounds issues
                    let coords = Coords { x: self.cursor.coords.x + i, y: self.cursor.coords.y };
                    self.grid[coords] = CharCell::Extension(self.cursor.coords,
                                                            self.cursor.text_style);
                }
                self.cursor.navigate(&mut self.grid, To(Right, 1, true));
            }
            CellData::Grapheme(c)   => {
                let width = c.width() as u32;
                self.grid[self.cursor.coords] = CharCell::grapheme(c, self.cursor.text_style);
                for i in 1..width {
                    //bounds issues
                    let coords = Coords { x: self.cursor.coords.x + i, y: self.cursor.coords.y };
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
            _                       => unimplemented!(),
        }
        self.grid_height = self.grid.height as u32;
    }

    pub fn move_cursor(&mut self, movement: Movement) {
        self.cursor.navigate(&mut self.grid, movement);
        self.grid_height = self.grid.height as u32;
    }

    pub fn scroll(&mut self, movement: Movement) {
        if let Some((n, dir)) = movement.as_direction() {
            self.grid.scroll(n as usize, dir)
        }
    }

    pub fn erase(&mut self, area: Area) {
        self.in_area(area, |grid, coords| grid[coords].empty());
    }

    pub fn insert_blank_at(&mut self, n: u32) {
        let vector = Vector::new(self.cursor.coords, ToEdge(Right), self.grid.bounds()).skip(1)
                            .collect::<Vec<_>>();
        for coords in vector.into_iter().rev().skip(n as usize) {
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
        for coords in region.iter().rev().skip(n as usize * self.grid.width) {
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

    fn in_area<F>(&mut self, area: Area, f: F)
    where F: Fn(&mut Grid<CharCell>, Coords) {
        let width = self.grid.width as u32;
        let height = self.grid.height as u32;
        match area {
            CursorCell                  => f(&mut self.grid, self.cursor.coords),
            CursorRow                   => {
                for coords in Region::new(0, self.cursor.coords.y, 0, self.cursor.coords.y + 1) {
                    f(&mut self.grid, coords);
                }
            }
            CursorColumn                => {
                for coords in Region::new(self.cursor.coords.x, 0, self.cursor.coords.x + 1, 0) {
                    f(&mut self.grid, coords);
                }
            }
            CursorTo(movement)          => {
                for coords in Vector::new(self.cursor.coords, movement, self.grid.bounds()) {
                    f(&mut self.grid, coords);
                }
            }
            CursorBound(coords)         => {
                for coords in Region::new(self.cursor.coords.x, self.cursor.coords.y,
                                          coords.x, coords.y) {
                    f(&mut self.grid, coords);
                }
            }
            BelowCursor(true)           => {
                for coords in Region::new(0, self.cursor.coords.y, width, height) {
                    f(&mut self.grid, coords);
                }
            }
            BelowCursor(false)          => {
                if self.cursor.coords.y == height - 1 { return; }
                for coords in Region::new(0, self.cursor.coords.y + 1, width, height) {
                    f(&mut self.grid, coords);
                }
            }
            WholeScreen                 => {
                for coords in self.grid.bounds() {
                    f(&mut self.grid, coords);
                }
            }
            Bound(region)               => {
                for coords in region {
                    f(&mut self.grid, coords);
                }
            }
            Rows(top, bottom)           => {
                for coords in Region::new(0, top, width, cmp::min(height, bottom)) {
                    f(&mut self.grid, coords);
                }
            }
            Columns(left, right)        => {
                for coords in Region::new(left, 0, cmp::min(width, right), height) {
                    f(&mut self.grid, coords);
                }
            }
        };
    }

}

impl<'a> IntoIterator for &'a CharGrid {
    type IntoIter = <&'a Grid<CharCell> as IntoIterator>::IntoIter;
    type Item = &'a CharCell;
    fn into_iter(self) -> Self::IntoIter {
        self.grid.into_iter()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use cfg;
    use datatypes::{CellData, Coords, Direction, Movement};
    use screen::{CharCell, Cursor, Grid, Styles};

    fn run_test<F: Fn(CharGrid, u32)>(test: F) {
        test(CharGrid::new(10, 10, false, false), 10);
        test(CharGrid::new(10, 10, false, true), 11);
    }

    #[test]
    fn write() {
        run_test(|mut grid, _| {
            for c in vec![
                CellData::Char('Q'),
                CellData::Grapheme(String::from("E\u{301}")),
                CellData::Char('E'),
                CellData::ExtensionChar('\u{301}'),
            ].into_iter() { grid.write(c); }
            assert_eq!(grid.grid[Coords {x:0, y:0}],
                       CharCell::character('Q', Styles::default()));
            assert_eq!(grid.grid[Coords {x:1, y:0}],
                       CharCell::grapheme(String::from("E\u{301}"), Styles::default()));
            assert_eq!(grid.grid[Coords {x:2, y:0}],
                       CharCell::grapheme(String::from("E\u{301}"), Styles::default()));
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
                (Movement::Tab(Direction::Right, 1, false), Coords{x:cfg::TAB_STOP, y:9}),
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
            assert_eq!(grid.grid[Coords {x:0, y:0}], CharCell::character('A', Styles::default()));
            assert_eq!(grid.grid[Coords {x:1, y:0}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:2, y:0}], CharCell::character('B', Styles::default()));
            assert_eq!(grid.grid[Coords {x:3, y:0}], CharCell::character('C', Styles::default()));
            assert_eq!(grid.grid[Coords {x:4, y:0}], CharCell::character('D', Styles::default()));
            assert_eq!(grid.grid[Coords {x:5, y:0}], CharCell::character('E', Styles::default()));
            grid.move_cursor(Movement::NextLine(1));
            grid.insert_blank_at(2);
            assert_eq!(grid.grid[Coords {x:0, y:1}], CharCell::character('1', Styles::default()));
            assert_eq!(grid.grid[Coords {x:1, y:1}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:2, y:1}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:3, y:1}], CharCell::character('2', Styles::default()));
            assert_eq!(grid.grid[Coords {x:4, y:1}], CharCell::character('3', Styles::default()));
            assert_eq!(grid.grid[Coords {x:5, y:1}], CharCell::character('4', Styles::default()));
            assert_eq!(grid.grid[Coords {x:6, y:1}], CharCell::character('5', Styles::default()));
            grid.move_cursor(Movement::NextLine(1));
            grid.insert_blank_at(3);
            assert_eq!(grid.grid[Coords {x:0, y:2}], CharCell::character('!', Styles::default()));
            assert_eq!(grid.grid[Coords {x:1, y:2}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:2, y:2}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:3, y:2}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:4, y:2}], CharCell::character('@', Styles::default()));
            assert_eq!(grid.grid[Coords {x:5, y:2}], CharCell::character('#', Styles::default()));
            assert_eq!(grid.grid[Coords {x:6, y:2}], CharCell::character('$', Styles::default()));
            assert_eq!(grid.grid[Coords {x:7, y:2}], CharCell::character('%', Styles::default()));
        })
    }

    #[test]
    fn remove_at() {
        run_test(|mut grid, _| {
            setup(&mut grid);
            grid.remove_at(1);
            assert_eq!(grid.grid[Coords {x:0, y:0}], CharCell::character('B', Styles::default()));
            assert_eq!(grid.grid[Coords {x:1, y:0}], CharCell::character('C', Styles::default()));
            assert_eq!(grid.grid[Coords {x:2, y:0}], CharCell::character('D', Styles::default()));
            assert_eq!(grid.grid[Coords {x:3, y:0}], CharCell::character('E', Styles::default()));
            assert_eq!(grid.grid[Coords {x:4, y:0}], CharCell::default());
            grid.move_cursor(Movement::NextLine(1));
            grid.remove_at(2);
            assert_eq!(grid.grid[Coords {x:0, y:1}], CharCell::character('3', Styles::default()));
            assert_eq!(grid.grid[Coords {x:1, y:1}], CharCell::character('4', Styles::default()));
            assert_eq!(grid.grid[Coords {x:2, y:1}], CharCell::character('5', Styles::default()));
            assert_eq!(grid.grid[Coords {x:3, y:1}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:4, y:1}], CharCell::default());
            grid.move_cursor(Movement::NextLine(1));
            grid.remove_at(3);
            assert_eq!(grid.grid[Coords {x:0, y:2}], CharCell::character('$', Styles::default()));
            assert_eq!(grid.grid[Coords {x:1, y:2}], CharCell::character('%', Styles::default()));
            assert_eq!(grid.grid[Coords {x:2, y:2}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:3, y:2}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:4, y:2}], CharCell::default());
        })
    }

    #[test]
    fn insert_rows_at() {
        run_test(|mut grid, _| {
            setup(&mut grid);
            grid.insert_rows_at(2, false);
            assert_eq!(grid.grid[Coords {x:0, y:1}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:1, y:1}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:2, y:1}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:3, y:1}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:4, y:1}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:0, y:2}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:1, y:2}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:2, y:2}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:3, y:2}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:4, y:2}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:0, y:3}], CharCell::character('1', Styles::default()));
            assert_eq!(grid.grid[Coords {x:1, y:3}], CharCell::character('2', Styles::default()));
            assert_eq!(grid.grid[Coords {x:2, y:3}], CharCell::character('3', Styles::default()));
            assert_eq!(grid.grid[Coords {x:3, y:3}], CharCell::character('4', Styles::default()));
            assert_eq!(grid.grid[Coords {x:4, y:3}], CharCell::character('5', Styles::default()));
            grid.insert_rows_at(3, true);
            assert_eq!(grid.grid[Coords {x:0, y:0}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:1, y:0}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:2, y:0}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:3, y:0}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:4, y:0}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:0, y:1}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:1, y:1}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:2, y:1}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:3, y:1}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:4, y:1}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:0, y:2}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:1, y:2}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:2, y:2}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:3, y:2}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:4, y:2}], CharCell::default());
            assert_eq!(grid.grid[Coords {x:0, y:3}], CharCell::character('A', Styles::default()));
            assert_eq!(grid.grid[Coords {x:1, y:3}], CharCell::character('B', Styles::default()));
            assert_eq!(grid.grid[Coords {x:2, y:3}], CharCell::character('C', Styles::default()));
            assert_eq!(grid.grid[Coords {x:3, y:3}], CharCell::character('D', Styles::default()));
            assert_eq!(grid.grid[Coords {x:4, y:3}], CharCell::character('E', Styles::default()));
        })
    }

    #[test]
    fn remove_rows_at() {
        run_test(|mut grid, _| {
            setup(&mut grid);
            grid.remove_rows_at(2, true);
            assert_eq!(grid.grid[Coords {x:0, y:0}], CharCell::character('!', Styles::default()));
            assert_eq!(grid.grid[Coords {x:1, y:0}], CharCell::character('@', Styles::default()));
            assert_eq!(grid.grid[Coords {x:2, y:0}], CharCell::character('#', Styles::default()));
            assert_eq!(grid.grid[Coords {x:3, y:0}], CharCell::character('$', Styles::default()));
            assert_eq!(grid.grid[Coords {x:4, y:0}], CharCell::character('%', Styles::default()));
        })
    }

}
