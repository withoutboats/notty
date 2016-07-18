use datatypes::{Coords, Region};
use terminal::{CharData, CellData, UseStyles};
use terminal::interfaces::{WriteableGrid, WriteableCell};

impl CharData for char {
    fn write<T>(&self, coords: Coords, styles: UseStyles, grid: &mut T) -> Coords
    where T: WriteableGrid, T::Cell: WriteableCell {
        if let Some(cell) = grid.writeable(coords) {
            cell.write(CellData::Char(*self), styles);
        }
        coords
    }

    #[cfg(any(debug_assertions, test))]
    fn repr(&self) -> String {
        self.to_string()
    }
}

pub struct WideChar(pub char, pub u32);

impl WideChar {
    pub fn new(ch: char, width: u32) -> WideChar {
        WideChar(ch, width)
    }
}

impl CharData for WideChar {
    fn write<T>(&self, coords: Coords, styles: UseStyles, grid: &mut T) -> Coords
    where T: WriteableGrid, T::Cell: WriteableCell {
        let coords = grid.best_fit_for_region(Region::new(coords.x, coords.y, coords.x + self.1, coords.y + 1));
        if let Some(cell) = grid.writeable(coords) {
            cell.write(CellData::Char(self.0), styles);
        }
        for extension_coords in (1..self.1).map(|i| Coords { x: coords.x + i, ..coords }) {
            if let Some(cell) = grid.writeable(extension_coords) {
                cell.write(CellData::Extension(coords), styles)
            }
        }
        Coords { x: coords.x + self.1 - 1, y: coords.y }
    }

    #[cfg(any(debug_assertions, test))]
    fn repr(&self) -> String {
        self.0.to_string()
    }
}

pub struct CharExtender(pub char);

impl CharExtender {
    pub fn new(ch: char) -> CharExtender {
        CharExtender(ch)
    }
}

impl CharData for CharExtender {
    fn write<T>(&self, coords: Coords, styles: UseStyles, grid: &mut T) -> Coords
    where T: WriteableGrid, T::Cell: WriteableCell {
        match grid.find_cell_to_extend(coords) {
            Some(coords)    => {
                if let Some(cell) = grid.writeable(coords) {
                    cell.extend(self.0, styles);
                }
                coords
            }
            None            => {
                if let Some(cell) = grid.writeable(coords) {
                    cell.write(CellData::Char(self.0), styles);
                }
                coords
            }
        }
    }

    #[cfg(any(debug_assertions, test))]
    fn repr(&self) -> String {
        self.0.to_string()
    }
}
