use cfg;
use datatypes::{Coords, Movement};
use datatypes::Direction::*;
use datatypes::Movement::*;
use screen::{CharCell, Grid, Styles};

#[derive(Copy, Clone)]
pub struct Cursor {
    pub coords: Coords,
    pub style: Styles,
    pub text_style: Styles,
}

impl Cursor {
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
        let mut coords = grid.bounds().move_within(self.coords, movement);
        if let CharCell::Extension(source_coords, _) = grid[coords] {
            loop {
                match movement.direction(self.coords) {
                    Right | Down if source_coords.y < coords.y || source_coords.x < coords.x => {
                        coords = grid.bounds().move_within(coords, To(Right, 1, true));
                        println!("{:?}", coords);
                        if !grid[coords].is_char_extension() {
                            self.coords = coords;
                            break;
                        }
                    }
                    _   => {
                        self.coords = source_coords;
                        break;
                    }
                }
            }
        } else { self.coords = coords };
    }
}

impl Default for Cursor {
    fn default() -> Cursor {
        Cursor {
            coords: Coords::default(),
            style: Styles {
                fg_color: cfg::CURSOR_COLOR,
                ..Styles::default()
            },
            text_style: Styles::default(),
        }
    }
}
