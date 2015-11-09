use cfg;
use datatypes::{Coords, Direction, Movement, Region};
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
        self.coords = Region::new(0, 0, grid.width as u32, grid.height as u32)
                            .move_within(self.coords, movement);
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
