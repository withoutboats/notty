use datatypes::CoordsIter;
use terminal::interfaces::CellGrid;
use super::{CharGrid, CharCell};

pub struct Cells<'a, G: 'a> {
    char_grid: &'a CharGrid<G>,
    iter: CoordsIter,
}

impl<'a, G> Cells<'a, G> {
    pub fn new(grid: &'a CharGrid<G>) -> Cells<'a, G> {
        Cells {
            char_grid: grid,
            iter: CoordsIter::from_region(grid.view.bounds()),
        }
    }
}

impl<'a, G: CellGrid<Cell=CharCell>> Iterator for Cells<'a, G> {
    type Item = &'a CharCell;

    fn next(&mut self) -> Option<&'a CharCell> {
        self.iter.next().map(|coords| &self.char_grid[coords])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, G: CellGrid<Cell=CharCell>> DoubleEndedIterator for Cells<'a, G> {
    fn next_back(&mut self) -> Option<&'a CharCell> {
        self.iter.next_back().map(|coords| &self.char_grid[coords])
    }
}

impl<'a, G: CellGrid<Cell=CharCell>> ExactSizeIterator for Cells<'a, G> { }
