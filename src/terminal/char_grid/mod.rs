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
use std::ops::{Index, Deref, DerefMut};

use datatypes::{Area, Coords, GridSettings, CoordsIter, Direction, Movement, Style, move_within};

use terminal::UseStyles;
use terminal::interfaces::*;

mod cell;
mod cursor;
mod grid;
mod tooltip;
mod view;
mod writers;

pub use self::cell::{CharCell, CellData, ImageData, EMPTY_CELL};
pub use self::cursor::Cursor;
pub use self::tooltip::Tooltip;
pub use self::writers::*;

use self::grid::Grid;
use self::view::View;

const RIGHT_ONE: Movement = Movement::To(Direction::Right, 1, true);
const TO_RIGHT_EDGE: Area = Area::CursorTo(Movement::ToEdge(Direction::Right));

pub struct CharGrid<G=Grid<CharCell>> {
    grid: G,
    cursor: Cursor,
    view: View,
    tooltips: HashMap<Coords, Tooltip>,
    text_styles: UseStyles,
}

// Public methods

impl<G: CellGrid + WriteableGrid> CharGrid<G> where <G as WriteableGrid>::Cell: WriteableCell {
    pub fn write<C: CharData>(&mut self, data: &C) {
        let coords = data.write(self.cursor.coords,
                                self.text_styles,
                                &mut self.grid);
        self.cursor.coords = self.calculate_movement(coords, RIGHT_ONE);
        self.view.keep_within(self.cursor.coords);
    }
}

impl<T: CellGrid> CharGrid<T> {
    pub fn move_cursor(&mut self, movement: Movement) {
        self.cursor.coords = self.calculate_movement(self.cursor.coords, movement);
        self.view.keep_within(self.cursor.coords);
    }

    pub fn insert_blank_at(&mut self, n: u32) {
        let iter = self.iterate_over_area(TO_RIGHT_EDGE);
        let CharGrid { ref mut grid, ref view, .. } = * self;
        let iter = iter.rev().skip(n as usize)
                       .map(|coords| view.translate(coords));
        for coords in iter {
            grid.moveover(coords, Coords { x: coords.x + n, y: coords.y });
        }
    }

    pub fn remove_at(&mut self, n: u32) {
        let iter = self.iterate_over_area(TO_RIGHT_EDGE);
        let CharGrid { ref mut grid, ref view, .. } = *self;
        let iter = iter.take_while(|&Coords { x, .. }| x + n < view.width())
                       .map(|coords| view.translate(coords));
        for coords in iter {
            grid.moveover(Coords { x: coords.x + n, y: coords.y }, coords);
        }
    }

    pub fn insert_rows_at(&mut self, n: u32, include: bool) {
        let iter = self.iterate_over_area(Area::BelowCursor(include));
        let CharGrid { ref mut grid, ref view, .. } = *self;
        let iter = iter.rev().skip((n * view.width()) as usize)
                       .map(|coords| view.translate(coords));
        for coords in iter {
            grid.moveover(coords, Coords { x: coords.x, y: coords.y + n });
        }
    }

    pub fn remove_rows_at(&mut self, n: u32, include: bool) {
        let iter = self.iterate_over_area(Area::BelowCursor(include));
        let CharGrid { ref mut grid, ref view, .. } = *self;
        let iter = iter.take_while(|&Coords { y, .. }| y + n < view.height())
                       .map(|coords| view.translate(coords));
        for coords in iter {
            grid.moveover(Coords { x: coords.x, y: coords.y + n }, coords);
        }
    }
}

impl<T: CellGrid> CharGrid<T> where T::Cell: Cell {
    pub fn erase(&mut self, area: Area) {
        for coords in self.iterate_over_area(area) {
            self.grid.get_mut(coords).map(Cell::erase);
        }
    }
}

impl<T: CellGrid> CharGrid<T> where T::Cell: Styleable {
    pub fn set_style_in_area(&mut self, area: Area, style: Style) {
        for coords in self.iterate_over_area(area) {
            self.grid.get_mut(coords).map(|cell| cell.set_style(style));
        }
    }

    pub fn reset_styles_in_area(&mut self, area: Area) {
        for coords in self.iterate_over_area(area) {
            self.grid.get_mut(coords).map(Styleable::reset_style);
        }
    }
}

impl<T> CharGrid<T> {
    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
    }

    pub fn tooltip_at(&self, coords: Coords) -> Option<&Tooltip> {
        self.tooltips.get(&coords)
    }

    pub fn tooltip_at_mut(&mut self, coords: Coords) -> Option<&mut Tooltip> {
        self.tooltips.get_mut(&coords)
    }

    pub fn add_tooltip(&mut self, coords: Coords, tooltip: String) {
        self.tooltips.insert(coords, Tooltip::Basic(tooltip));
    }

    pub fn add_drop_down(&mut self, coords: Coords, options: Vec<String>) {
        self.tooltips.insert(coords, Tooltip::Menu { options: options, position: None });
    }

    pub fn remove_tooltip(&mut self, coords: Coords) {
        self.tooltips.remove(&coords);
    }
}

impl<T: ConstructGrid> ConstructGrid for CharGrid<T> {
    fn new(settings: GridSettings) -> CharGrid<T> {
        CharGrid {
            grid: T::new(settings),
            cursor: Cursor::default(),
            view: View::new(settings),
            tooltips: HashMap::new(),
            text_styles: UseStyles::default(),
        }
    }
}

impl<T: Resizeable> Resizeable for CharGrid<T> {
    fn resize_width(&mut self, width: u32) {
        self.view.resize_width(width);
        self.grid.resize_width(width);
    }

    fn resize_height(&mut self, height: u32) {
        self.view.resize_height(height);
        self.grid.resize_height(height);
    }
}

impl<T> Styleable for CharGrid<T> {
    fn styles(&self) -> &UseStyles {
        &self.text_styles
    }

    fn styles_mut(&mut self) -> &mut UseStyles {
        &mut self.text_styles
    }
}

impl<T: CellGrid<Cell=CharCell>> Index<Coords> for CharGrid<T> {
    type Output = CharCell;

    fn index(&self, coords: Coords) -> &CharCell {
        static DEFAULT_CELL: &'static CharCell = &EMPTY_CELL;
        self.grid.get(coords).unwrap_or(DEFAULT_CELL)
    }
}

impl<T> Deref for CharGrid<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.grid
    }
}

impl<T> DerefMut for CharGrid<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.grid
    }
}


// Private methods

impl<T> CharGrid<T> {
    fn iterate_over_area(&self, area: Area) -> CoordsIter {
        CoordsIter::from_area(area, self.cursor.coords, self.view.bounds())
    }
}

impl<T: CellGrid> CharGrid<T> {
    fn calculate_movement(&self, coords: Coords, movement: Movement) -> Coords {
        let new_coords = move_within(self.cursor.coords, movement, self.view.bounds());
        self.grid.move_out_of_extension(new_coords, movement.direction(coords))
    }
}
