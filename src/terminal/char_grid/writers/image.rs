use std::cell::RefCell;
use std::sync::Arc;

use mime::Mime;

use datatypes::{Coords, Region, CoordsIter, MediaPosition};
use terminal::{CellData, ImageData, UseStyles};
use terminal::interfaces::{CharData, WriteableGrid, WriteableCell};


pub struct Image {
    data: RefCell<Option<(Vec<u8>, Mime)>>,
    pos: MediaPosition,
    width: u32,
    height: u32,
}

impl Image {
    pub fn new(data: Vec<u8>, mime: Mime, pos: MediaPosition, w: u32, h: u32) -> Image {
        Image {
            data: RefCell::new(Some((data, mime))),
            pos: pos,
            width: w,
            height: h,
        }
    }
}

impl CharData for Image {
    fn write<T>(&self, coords: Coords, styles: UseStyles, grid: &mut T) -> Coords
    where T: WriteableGrid, T::Cell: WriteableCell {
        if let Some((data, mime)) = self.data.borrow_mut().take() {
            let coords = grid.best_fit_for_region(Region::new(coords.x, coords.y, coords.x + self.width, coords.y + self.height));
            if let Some(cell) = grid.writeable(coords) {
                let image = CellData::Image {
                    data: Arc::new(ImageData {
                        data: data,
                        coords: coords,
                    }),
                    mime: mime,
                    pos: self.pos,
                    width: self.width,
                    height: self.height,
                };
                cell.write(image, styles);
            }
            let iter = CoordsIter::from(Region::new(coords.x, coords.y, self.width, self.height));
            for extension_coords in iter.skip(1) {
                if let Some(cell) = grid.writeable(extension_coords) {
                    cell.write(CellData::Extension(coords), styles);
                }
            }
            Coords { x: coords.x + self.width - 1, y: coords.y }
        } else { coords }
    }

    #[cfg(any(debug_assertions, test))]
    fn repr(&self) -> String {
        String::from("IMAGE")
    }
}
