extern crate cairo;
extern crate gdk;
extern crate gdk_pixbuf_sys as pixbuf;
extern crate gio_sys as gio;
extern crate itertools;
extern crate libc;
extern crate notty;
extern crate pangocairo;

mod image_renderer;
mod text_renderer;

use itertools::Itertools;

use notty::cfg::CONFIG;
use notty::datatypes::Color;
use notty::terminal::{CharCell, Terminal};

use self::image_renderer::ImageRenderer;
use self::text_renderer::TextRenderer;

pub struct Renderer {
    char_w: f64,
    char_h: f64,
    scroll: u32,
}

impl Renderer {
    pub fn new(canvas: &cairo::Context, scroll: u32) -> Renderer {
        let (char_w, char_h) = char_dimensions(canvas);
        Renderer { char_w: char_w, char_h: char_h, scroll: scroll }
    }

    pub fn reset_dimensions(&self, terminal: &mut Terminal, x_pix: u32, y_pix: u32) {
        terminal.set_winsize(x_pix / self.char_w as u32, y_pix / self.char_h as u32)
                .unwrap_or_else(|e| panic!("{}", e))
    }

    pub fn draw(&self, terminal: &Terminal, canvas: &cairo::Context) {
        let Color(r,g,b) = CONFIG.bg_color;
        canvas.set_source_rgb(color(r), color(g), color(b));
        canvas.paint();

        let col_n = terminal.grid_width as usize;
        let rows = terminal.into_iter().skip(self.scroll as usize * col_n).chunks_lazy(col_n);

        for (y_pos, row) in rows.into_iter().enumerate() {
            let y_pix = y_pix(canvas, y_pos);
            let mut text = TextRenderer::new(0.0, y_pix);
            for (x_pos, cell) in row.enumerate() {
                match cell {
                    &CharCell::Empty(style)                             => text.push(' ', style),
                    &CharCell::Char(ch, style)                          => text.push(ch, style),
                    &CharCell::Grapheme(ref s, style)                   => text.push_str(s, style),
                    &CharCell::Extension(..)                            => { }
                    &CharCell::Image { ref data, width, style, ..}      => {
                        if (x_pos + width as usize) < col_n {
                            text.draw(canvas);
                            text = TextRenderer::new(x_pix(canvas, x_pos + width as usize), y_pix);
                        }
                        ImageRenderer::new(&data, style, x_pix(canvas, x_pos), y_pix).draw(canvas);
                    }
                }
            }
            text.draw(canvas);
        }
    }
}

fn char_dimensions(canvas: &cairo::Context) -> (f64, f64) {
    let f_extents = canvas.font_extents();
    (f_extents.max_x_advance, f_extents.height)
}

fn color(byte: u8) -> f64 {
    byte as f64 / 255.0
}

fn x_pix(canvas: &cairo::Context, position: usize) -> f64 {
    position as f64 * (canvas.font_extents().max_x_advance)
}

fn y_pix(canvas: &cairo::Context, position: usize) -> f64 {
    let f_extents = canvas.font_extents();
    position as f64 * (f_extents.height + f_extents.ascent + f_extents.descent)
}
