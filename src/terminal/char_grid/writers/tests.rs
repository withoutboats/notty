pub use datatypes::{Coords, Region};
pub use terminal::{CellData, CharCell, UseStyles};
pub use terminal::interfaces::*;

pub use super::*;

pub const COORDS: Coords = Coords { x: 7, y: 19 };
pub const STYLES: UseStyles = ::terminal::styles::DEFAULT_STYLES;

mod test_char {
    use super::*;
    const CHAR: char = 'Q';
    const DATA: CellData = CellData::Char(CHAR);

    struct Grid(Cell);
    struct Cell;

    impl WriteableGrid for Grid {
        type Cell = Cell;
        fn writeable(&mut self, coords: Coords) -> Option<&mut Cell> {
            assert_eq!(coords, COORDS);
            Some(&mut self.0)
        }
        fn best_fit_for_region(&self, _: Region) -> Coords { unreachable!() }
        fn find_cell_to_extend(&self, _: Coords) -> Option<Coords> { unreachable!() }
    }

    impl WriteableCell for Cell {
        fn write(&mut self, data: CellData, styles: UseStyles) {
            assert_eq!(data, DATA);
            assert_eq!(styles, STYLES);
        }
        fn extend(&mut self, _: char, _: UseStyles) { unreachable!() }
        fn is_extendable(&self) -> bool { unreachable!() }
        fn source(&self) -> Option<Coords> { unreachable!() }
    }

    #[test]
    fn char_write() {
        assert_eq!(CHAR.write(COORDS, STYLES, &mut Grid(Cell)), COORDS);
    }
}

mod test_wide_char {
    use super::*;
    const CHAR: char = 'R';
    const WIDTH: u32 = 2;
    const WIDE_CHAR: WideChar = WideChar(CHAR, WIDTH);
    const REGION: Region = Region {
        left: COORDS.x,
        top: COORDS.y,
        right: COORDS.x + WIDTH,
        bottom: COORDS.y + 1
    };
    const BEST_FIT_COORDS: Coords = Coords { x: 1, y: 1 };
    const FINAL_COORDS: Coords = Coords { x: BEST_FIT_COORDS.x + 1, ..BEST_FIT_COORDS };

    struct Grid(Cell, Cell);
    enum Cell { Char, Extension }

    impl WriteableGrid for Grid {
        type Cell = Cell;
        fn writeable(&mut self, coords: Coords) -> Option<&mut Cell> {
            match (coords.x, coords.y) {
                (1, 1)  => Some(&mut self.0),
                (2, 1)  => Some(&mut self.1),
                _       => panic!("Passed incorrect coords to write_to: {:?}", coords),
            }
        }
        fn best_fit_for_region(&self, region: Region) -> Coords {
            assert_eq!(region, REGION);
            BEST_FIT_COORDS
        }
        fn find_cell_to_extend(&self, _: Coords) -> Option<Coords> { unreachable!() }
    }

    impl WriteableCell for Cell {
        fn write(&mut self, data: CellData, styles: UseStyles) {
            match *self {
                Cell::Char      => assert_eq!(data, CellData::Char(CHAR)),
                Cell::Extension => assert_eq!(data, CellData::Extension(BEST_FIT_COORDS)),
            }
            assert_eq!(styles, STYLES);
        }
        fn extend(&mut self, _: char, _: UseStyles) { unreachable!() }
        fn is_extendable(&self) -> bool { unreachable!() }
        fn source(&self) -> Option<Coords> { unreachable!() }
    }

    #[test]
    fn wide_char_write() {
        assert_eq!(WIDE_CHAR.write(COORDS, STYLES, &mut Grid(Cell::Char, Cell::Extension)), FINAL_COORDS);
    }
}

mod test_char_extender {
    pub use super::*;

    pub const CHAR: char = '$';
    pub const DATA: CellData = CellData::Char(CHAR);
    pub const CHAR_EXTENDER: CharExtender = CharExtender(CHAR);
    pub const COORDS_BEFORE: Coords = Coords { x: 0, y: 0 };

    mod extendable_cell {
        use super::*;

        const FINAL_COORDS: Coords = COORDS_BEFORE;

        struct Grid(Cell);
        struct Cell;

        impl WriteableGrid for Grid {
            type Cell = Cell;
            fn writeable(&mut self, coords: Coords) -> Option<&mut Cell> {
                assert_eq!(coords, COORDS_BEFORE);
                Some(&mut self.0)
            }
            fn best_fit_for_region(&self, _: Region) -> Coords { unreachable!() }
            fn find_cell_to_extend(&self, coords: Coords) -> Option<Coords> {
                assert_eq!(coords, COORDS);
                Some(COORDS_BEFORE)
            }
        }

        impl WriteableCell for Cell {
            fn write(&mut self, _: CellData, _: UseStyles) { unreachable!() }
            fn extend(&mut self, c: char, styles: UseStyles) {
                assert_eq!(c, CHAR);
                assert_eq!(styles, STYLES);
            }
            fn is_extendable(&self) -> bool { unreachable!() }
            fn source(&self) -> Option<Coords> { unreachable!() }
            
        }

        #[test]
        fn char_extender_write() {
            assert_eq!(CHAR_EXTENDER.write(COORDS, STYLES, &mut Grid(Cell)), FINAL_COORDS);
        }
    }

    mod non_extendable_cell {
        use super::*;

        const FINAL_COORDS: Coords = COORDS;

        struct Grid(Cell);
        struct Cell;

        impl WriteableGrid for Grid {
            type Cell = Cell;
            fn writeable(&mut self, coords: Coords) -> Option<&mut Cell> {
                assert_eq!(coords, COORDS);
                Some(&mut self.0)
            }
            fn best_fit_for_region(&self, _: Region) -> Coords { unreachable!() }
            fn find_cell_to_extend(&self, coords: Coords) -> Option<Coords> {
                assert_eq!(coords, COORDS);
                None
            }
        }

        impl WriteableCell for Cell {
            fn write(&mut self, data: CellData, styles: UseStyles) {
                assert_eq!(data, DATA);
                assert_eq!(styles, STYLES);
            }
            fn extend(&mut self, _: char, _: UseStyles) { unreachable!() }
            fn is_extendable(&self) -> bool { unreachable!() }
            fn source(&self) -> Option<Coords> { unreachable!() }
        }

        #[test]
        fn char_extender_write() {
            assert_eq!(CHAR_EXTENDER.write(COORDS, STYLES, &mut Grid(Cell)), FINAL_COORDS);
        }
    }
}

mod test_image {
    use super::*;

    use std::str::FromStr;
    use mime::Mime;
    use datatypes::MediaPosition;
    use terminal::image::{EncodedData, ImageFormat};

    // TODO what about when image is wider than grid is allowed to be?
    const FINAL_COORDS: Coords = Coords { x: BEST_FIT_COORDS.x + WIDTH - 1, ..BEST_FIT_COORDS };
    const DATA: &'static [u8] = &[0x0B, 0xEE, 0xFD, 0xAD];
    const MIME: &'static str = "image/jpeg";
    const IMAGE_FORMAT: ImageFormat = ImageFormat::Jpeg;
    const MEDIA_POSITION: MediaPosition = MediaPosition::Fill;
    const WIDTH: u32 = 5;
    const HEIGHT: u32 = 6;
    const REGION: Region = Region {
        left: COORDS.x,
        top: COORDS.y,
        right: COORDS.x + WIDTH,
        bottom: COORDS.y + HEIGHT,
    };
    const BEST_FIT_COORDS: Coords = Coords { x: 0, y: 0 };

    struct Grid(Cell, Cell);
    enum Cell { Image, Extension }

    impl WriteableGrid for Grid {
        type Cell = Cell;
        fn writeable(&mut self, coords: Coords) -> Option<&mut Cell> {
            match (coords.x, coords.y) {
                (0, 0)                              => Some(&mut self.0),
                (x, y) if x < WIDTH && y < HEIGHT   => Some(&mut self.1),
                _ => panic!("Passed incorrect coords to write_to: {:?}", coords),
            }
        }
        fn best_fit_for_region(&self, region: Region) -> Coords {
            assert_eq!(region, REGION);
            BEST_FIT_COORDS
        }
        fn find_cell_to_extend(&self, _: Coords) -> Option<Coords> { unreachable!() }
    }

    impl WriteableCell for Cell {
        fn write(&mut self, data: CellData, styles: UseStyles) {
            match *self {
                Cell::Image     => {
                    if let CellData::Image(image)  = data {
                        assert_eq!(image.pos(), MEDIA_POSITION);
                        assert_eq!(image.dims(), (WIDTH, HEIGHT));
                        image.decode(|data| {
                            assert_eq!(*data, EncodedData {
                                data: DATA.into(),
                                format: IMAGE_FORMAT,
                            });
                            vec![]
                        });
                    } else { panic!("instead of image, recieved: {:?}", data) }
                }
                Cell::Extension => assert_eq!(data, CellData::Extension(BEST_FIT_COORDS)),
            }
            assert_eq!(styles, STYLES);
        }
        fn extend(&mut self, _: char, _: UseStyles) { unreachable!() }
        fn is_extendable(&self) -> bool { unreachable!() }
        fn source(&self) -> Option<Coords> { unreachable!() }
    }

    #[test]
    fn image_write() {
        let data = Vec::from(DATA);
        let mime = Mime::from_str(MIME).unwrap();
        let image = Image::new(data, mime, MEDIA_POSITION, WIDTH, HEIGHT);
        assert_eq!(image.write(COORDS, STYLES, &mut Grid(Cell::Image, Cell::Extension)), FINAL_COORDS);
    }
}
