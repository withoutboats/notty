use datatypes::*;
use terminal::{CellData, UseStyles};

pub trait Styleable {
    fn styles(&self) -> &UseStyles;
    fn styles_mut(&mut self) -> &mut UseStyles;

    fn set_style(&mut self, style: Style) {
        self.styles_mut().update(style);
    }

    fn reset_style(&mut self) {
        *self.styles_mut() = UseStyles::default();
    }
}

pub trait Resizeable {
    fn dims(&self) -> (u32, u32);
    fn resize_width(&mut self, width: u32);
    fn resize_height(&mut self, height: u32);

    fn resize(&mut self, width: u32, height: u32) {
        self.resize_width(width);
        self.resize_height(height);
    }
}

pub trait ConstructGrid {
    fn new(settings: GridSettings) -> Self;
}

pub trait CharData: Send + 'static {
    fn write<T>(&self, coords: Coords, styles: UseStyles, grid: &mut T) -> Coords
    where T: WriteableGrid, T::Cell: WriteableCell;

    #[cfg(any(test, debug_assertions))]
    fn repr(&self) -> String {
        String::from("DATA")
    }
}

pub trait CellGrid {
    type Cell;
    fn get(&self, coords: Coords) -> Option<&Self::Cell>;
    fn get_mut(&mut self, coords: Coords) -> Option<&mut Self::Cell>;
    fn moveover(&mut self, from: Coords, to: Coords);
    fn move_out_of_extension(&self, coords: Coords, direction: Direction) -> Coords;
}

pub trait Cell: Styleable {
    fn is_extension(&self) -> bool;
    fn erase(&mut self);
}

pub trait WriteableGrid {
    type Cell;
    fn writeable(&mut self, coords: Coords) -> Option<&mut Self::Cell>;
    fn best_fit_for_region(&self, region: Region) -> Coords;
    fn find_cell_to_extend(&self, coords: Coords) -> Option<Coords>;
}

pub trait WriteableCell { 
    fn write(&mut self, data: CellData, styles: UseStyles);
    fn extend(&mut self, c: char, styles: UseStyles);
    fn is_extendable(&self) -> bool;
    fn source(&self) -> Option<Coords>;
}
