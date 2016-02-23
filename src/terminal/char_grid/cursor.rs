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
use cfg::Config;
use datatypes::{Coords, Movement, move_within};
use datatypes::Direction::*;
use datatypes::Movement::*;
use terminal::{CharCell, Grid, Styles};

#[derive(Copy, Clone)]
pub struct Cursor {
    pub coords: Coords,
    pub style: Styles,
    pub text_style: Styles,
    config: Config,
}

impl Cursor {

    pub fn new(config: Config) -> Cursor {
        Cursor {
            coords: Coords::default(),
            style: Styles::new(config),
            text_style: Styles::new(config),
            config: config,
        }
    }

    pub fn navigate(&mut self, grid: &mut Grid<CharCell>, movement: Movement) {
        match movement {
            IndexTo(Up, n) | PreviousLine(n) if n > self.coords.y => {
                let n = n - self.coords.y;
                grid.scroll(n as usize, Up);
            }
            IndexTo(Down, n) | NextLine(n) if self.coords.y + n >= grid.height as u32 => {
                let n = self.coords.y + n - grid.height as u32 + 1;
                grid.scroll(n as usize, Down);
            }
            IndexTo(Left, n) if n > self.coords.x => {
                let n = n - self.coords.x;
                grid.scroll(n as usize, Left);
            }
            IndexTo(Right, n) if self.coords.x + n >= grid.width as u32 => {
                let n = self.coords.x + n - grid.width as u32 + 1;
                grid.scroll(n as usize, Right);
            }
            _   => (),
        }
        let mut coords = move_within(self.coords, movement, grid.bounds(), self.config.tab_stop);

        if let CharCell::Extension(source, _) = grid[coords] {
            match movement {
                Position(_) => {
                    self.coords = source;
                    return;
                }
                ToBeginning => {
                    self.coords = Coords{x:0,y:0};
                    return;
                }
                ToEnd       => {
                    self.coords = Coords {x: (grid.width-1) as u32, y: (grid.height-1) as u32};
                    return;
                }
                _           => ()
            }
            if let Position(_) = movement {
            }
            match movement.direction(self.coords) {
                Up | Left                   => self.coords = source,
                dir @ Down | dir @ Right    => loop {
                    let next_coords = move_within(coords, To(dir, 1, false), grid.bounds(),
                                                  self.config.tab_stop);
                    if next_coords == coords { self.coords = source; return; }
                    if let CharCell::Extension(source2, _) = grid[next_coords] {
                        if source2 != source { self.coords = source2; return; }
                        else { coords = next_coords; }
                    } else { self.coords = next_coords; return; }
                }
            }
        } else { self.coords = coords };

    }

}

#[cfg(test)]
mod tests {

    use super::*;

    use datatypes::{Coords, Movement};
    use datatypes::Direction::*;
    use datatypes::Movement::*;
    use terminal::{Grid, CharCell, Styles};
    use cfg::Config;

    static MOVEMENTS: &'static [(Movement, Coords)] = &[
        (To(Left, 1, false), Coords {x: 1, y: 2}),
        (IndexTo(Up, 3), Coords {x: 2, y: 0}),
        (Position(Coords {x: 4, y: 4}), Coords {x: 4, y: 4})
    ];


    fn cursor() -> Cursor {
        Cursor { coords: Coords {x: 2, y: 2}, ..Cursor::new(Config::default()) }
    }

    #[test]
    fn navigate() {
        let default = CharCell::Empty(Styles::new(Config::default()));
        let mut grid = Grid::new(5, 5, default);
        for &(mov, coords) in MOVEMENTS {
            let mut cursor = cursor();
            cursor.navigate(&mut grid, mov);
            assert_eq!(cursor.coords, coords);
        }
    }

    #[test]
    fn navigate_and_scroll() {
        let default = CharCell::Empty(Styles::new(Config::default()));
        let mut grid = Grid::with_y_cap(5, 5, 10, default);
        for &(mov, coords) in MOVEMENTS {
            let mut cursor = cursor();
            cursor.navigate(&mut grid, mov);
            assert_eq!(cursor.coords, coords);
        }
        assert_eq!(grid.height, 6);
    }

    static MOVEMENTS_EXTENDED: &'static [(Coords, Movement, Coords)] = &[
        (Coords{x:1,y:1}, To(Right, 1, false), Coords{x:3,y:1}),
        (Coords{x:1,y:1}, To(Down, 1, false), Coords{x:1,y:3}),
        (Coords{x:3,y:1}, To(Left, 1, false), Coords{x:1,y:1}),
        (Coords{x:1,y:3}, To(Up, 1, false), Coords{x:1,y:1}),
        (Coords{x:1,y:1}, NextLine(1), Coords{x:0,y:2}),
        (Coords{x:0,y:0}, Position(Coords{x:2,y:2}), Coords{x:1,y:1}),
        (Coords{x:0,y:1}, To(Right, 2, false), Coords{x:3,y:1}),
        (Coords{x:1,y:0}, To(Down, 2, false), Coords{x:1,y:3}),
        (Coords{x:4,y:1}, To(Left, 2, false), Coords{x:1,y:1}),
        (Coords{x:1,y:4}, To(Up, 2, false), Coords{x:1,y:1}),
    ];

    #[test]
    fn navigate_around_extended_cells() {
        let default = CharCell::Empty(Styles::new(Config::default()));
        let mut grid = Grid::new(5, 5, default);
        let config = Config::default();
        grid[Coords{x:2,y:1}] = CharCell::Extension(Coords{x:1,y:1}, Styles::new(config));
        grid[Coords{x:1,y:2}] = CharCell::Extension(Coords{x:1,y:1}, Styles::new(config));
        grid[Coords{x:2,y:2}] = CharCell::Extension(Coords{x:1,y:1}, Styles::new(config));
        for &(init, mov, end) in MOVEMENTS_EXTENDED {
            let mut cursor = Cursor { coords: init, ..Cursor::new(Config::default()) };
            cursor.navigate(&mut grid, mov);
            assert_eq!(cursor.coords, end);
        }
    }

    static MOVEMENTS_EXTENDED_AT_BORDER: &'static [(Coords, Movement, Coords)] = &[
        (Coords{x:0,y:1}, To(Right, 1, false), Coords{x:0,y:1}),
        (Coords{x:0,y:1}, To(Down, 1, false), Coords{x:0,y:3}),
        (Coords{x:0,y:3}, To(Up, 1, false), Coords{x:0,y:1}),
        (Coords{x:0,y:1}, NextLine(1), Coords{x:0,y:3}),
        (Coords{x:0,y:3}, PreviousLine(1), Coords{x:0,y:1}),
        (Coords{x:0,y:0}, To(Down, 2, false), Coords{x:0,y:3}),
        (Coords{x:0,y:4}, To(Up, 2, false), Coords{x:0,y:1}),
        (Coords{x:1,y:4}, To(Up, 2, false), Coords{x:0,y:1}),
    ];

    #[test]
    fn navigate_around_extended_at_border() {
        let default = CharCell::Empty(Styles::new(Config::default()));
        let mut grid = Grid::new(2, 5, default);
        let config = Config::default();
        grid[Coords{x:1,y:1}] = CharCell::Extension(Coords{x:0,y:1}, Styles::new(config));
        grid[Coords{x:0,y:2}] = CharCell::Extension(Coords{x:0,y:1}, Styles::new(config));
        grid[Coords{x:1,y:2}] = CharCell::Extension(Coords{x:0,y:1}, Styles::new(config));
        for &(init, mov, end) in MOVEMENTS_EXTENDED_AT_BORDER {
            let mut cursor = Cursor { coords: init, ..Cursor::new(Config::default()) };
            cursor.navigate(&mut grid, mov);
            assert_eq!(cursor.coords, end);
        }
    }

}
