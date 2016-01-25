pub use super::*;
pub use super::grid_hierarchy::*;
pub use terminal::{CharCell, CharGrid, Styles};
pub use datatypes::{CellData, Movement};

mod one {
    use super::*;
    // This tests a screen with only one grid, filled with the char '0'.
    //
    //      0
    //
    //    0 0 0 0 0 0 0 0
    //    0 0 0 0 0 0 0 0
    //    0 0 0 0 0 0 0 0
    //    0 0 0 0 0 0 0 0
    //    0 0 0 0 0 0 0 0
    //    0 0 0 0 0 0 0 0
    //    0 0 0 0 0 0 0 0
    //    0 0 0 0 0 0 0 0
    //
    fn setup_screen() -> Screen {
        let mut s = Screen::new(8, 8);
        super::fill_grid(&mut s.active_grid.1, '0');
        s
    }

    #[test]
    fn is_setup_correctly() {
        for c in &setup_screen() {
            assert_eq!(*c, CharCell::character('0', Styles::default()));
        }
    }
}

mod two {
    use super::*;
    // This tests a screen with two grids, split evenly down the middle.
    //
    //      0
    //      | \ 
    //      10 11
    //
    //    0 0 0 0 1 1 1 1
    //    0 0 0 0 1 1 1 1
    //    0 0 0 0 1 1 1 1
    //    0 0 0 0 1 1 1 1
    //    0 0 0 0 1 1 1 1
    //    0 0 0 0 1 1 1 1
    //    0 0 0 0 1 1 1 1
    //    0 0 0 0 1 1 1 1
    //
    fn setup_screen() -> Screen {
        let mut s = Screen::new(8, 8);
        super::fill_grid(&mut s.active_grid.1, '0');
        s.split_vertical(4, SaveGrid::Left, 10, 11);
        s.switch(11);
        super::fill_grid(&mut s.active_grid.1, '1');
        s
    }

    #[test]
    fn is_setup_correctly() {
        for (n, c) in (&setup_screen()).into_iter().enumerate() {
            assert_eq!(*c, CharCell::character(if (n % 8) / 4 == 0 { '0' } else { '1' },
                                              Styles::default()));
        }
    }

    #[test]
    fn remove_grid_one() {
        let mut s = setup_screen();
        s.switch(0);
        s.remove(11);
        super::fill_grid(&mut s.active_grid.1, '0');
        for c in &s {
            assert_eq!(*c, CharCell::character('0', Styles::default()));
        }
    }
}

mod three {
    use super::*;
    // This tests a screen with three grids, one over the bottom 3 rows and the other two split
    // evenly down the middle.
    //
    //      0
    //      | \
    //      10 12
    //      | \
    //      20 21
    //
    //    0 0 0 0 1 1 1 1
    //    0 0 0 0 1 1 1 1
    //    0 0 0 0 1 1 1 1
    //    0 0 0 0 1 1 1 1
    //    0 0 0 0 1 1 1 1
    //    2 2 2 2 2 2 2 2
    //    2 2 2 2 2 2 2 2
    //    2 2 2 2 2 2 2 2
    //
    fn setup_screen() -> Screen {
        let mut s = Screen::new(8, 8);
        super::fill_grid(&mut s.active_grid.1, '0');
        s.split_horizontal(5, SaveGrid::Left, 10, 12);
        s.switch(12);
        super::fill_grid(&mut s.active_grid.1, '2');
        s.switch(10);
        s.split_vertical(4, SaveGrid::Left, 20, 21);
        s.switch(21);
        super::fill_grid(&mut s.active_grid.1, '1');
        s
    }

    #[test]
    fn is_setup_correctly() {
        for (n, c) in (&setup_screen()).into_iter().enumerate() {
            if n < 40 {
                assert_eq!(*c, CharCell::character(if (n % 8) / 4 == 0 { '0' } else { '1' },
                                                  Styles::default()));
            } else {
                assert_eq!(*c, CharCell::character('2', Styles::default()));
            }
        }
    }

    #[test]
    fn remove_grid_two() {
        let mut s = setup_screen();
        s.switch(0);
        s.remove(12);
        s.switch(20);
        super::fill_grid(&mut s.active_grid.1, '0');
        s.switch(21);
        super::fill_grid(&mut s.active_grid.1, '1');
        for (n, c) in (&s).into_iter().enumerate() {
            assert_eq!(*c, CharCell::character(if (n % 8) / 4 == 0 { '0' } else { '1' },
                                               Styles::default()));
        }
    }
}

mod four {
    use super::*;
    // This tests a screen with four grids, like so:
    //        0
    //       / \
    //     10   11
    //    / |   | \
    //  20 21   22 23
    //
    //    0 0 0 0 1 1 1 1
    //    0 0 0 0 1 1 1 1
    //    0 0 0 0 1 1 1 1
    //    0 0 0 0 1 1 1 1
    //    0 0 0 0 1 1 1 1
    //    2 2 2 2 2 2 2 2
    //    2 2 2 2 2 2 2 2
    //    3 3 3 3 3 3 3 3
    //
    fn setup_screen() -> Screen {
        let mut s = Screen::new(8, 8);
        super::fill_grid(&mut s.active_grid.1, '0');
        s.split_horizontal(5, SaveGrid::Left, 10, 11);
        s.switch(11);
        super::fill_grid(&mut s.active_grid.1, '2');
        s.split_horizontal(2, SaveGrid::Left, 22, 23);
        s.switch(23);
        super::fill_grid(&mut s.active_grid.1, '3');
        s.switch(10);
        s.split_vertical(4, SaveGrid::Left, 20, 21);
        s.switch(21);
        super::fill_grid(&mut s.active_grid.1, '1');
        s
    }

    #[test]
    fn is_setup_correctly() {
        for (n, c) in (&setup_screen()).into_iter().enumerate() {
            if n < 40 {
                assert_eq!(*c, CharCell::character(if (n % 8) / 4 == 0 { '0' } else { '1' },
                                                  Styles::default()));
            } else if n < 56 {
                assert_eq!(*c, CharCell::character('2', Styles::default()));
            } else {
                assert_eq!(*c, CharCell::character('3', Styles::default()));
            }
        }
    }

    #[test]
    fn remove_grid_two() {
        let mut s = setup_screen();
        s.switch(0);
        s.remove(22);
        for (n, c) in (&s).into_iter().enumerate() {
            if n < 40 {
                assert_eq!(*c, CharCell::character(if (n % 8) / 4 == 0 { '0' } else { '1' },
                                                   Styles::default()));
            } else {
                assert_eq!(*c, CharCell::character('3', Styles::default()));
            }
        }
    }
}

fn fill_grid(grid: &mut CharGrid, c: char) {
    grid.move_cursor(Movement::ToBeginning);
    for _ in 0..(grid.grid_width * grid.grid_height) {
        grid.write(CellData::Char(c));
    }
}
