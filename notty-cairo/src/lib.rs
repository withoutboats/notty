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

use gdk::glib::translate::ToGlibPtr;

use itertools::Itertools;

use notty::cfg::CONFIG;
use notty::datatypes::Color;
use notty::terminal::{CharCell, Terminal};

use pangocairo::wrap::{PangoLayout, PangoAttrList};

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
            let y_pix = self.y_pixels(y_pos as u32);
            let mut text = TextRenderer::new(0.0, y_pix);
            for (x_pos, cell) in row.enumerate() {
                match cell {
                    &CharCell::Empty(style)                             => text.push(' ', style),
                    &CharCell::Char(ch, style)                          => text.push(ch, style),
                    &CharCell::Grapheme(ref s, style)                   => text.push_str(s, style),
                    &CharCell::Extension(..)                            => { }
                    &CharCell::Image { ref data, width, height, pos, ..} => {
                        let x_pix = self.x_pixels(x_pos as u32);
                        if (x_pos + width as usize) < col_n {
                            text.draw(canvas);
                            text = TextRenderer::new(x_pix, y_pix);
                        }
                        let w_pix = self.x_pixels(width);
                        let h_pix = self.y_pixels(height);
                        ImageRenderer::new(&data, x_pix, y_pix, w_pix, h_pix, pos).draw(canvas);
                    }
                }
            }
            text.draw(canvas);
        }
    }

    fn x_pixels(&self, x: u32) -> f64 {
        self.char_w * (x as f64)
    }

    fn y_pixels(&self, y: u32) -> f64 {
        self.char_h * (y as f64)
    }
}

fn char_dimensions(canvas: &cairo::Context) -> (f64, f64) {
    //save the canvas position
    let (x_save, y_save) = canvas.get_current_point();
    let string = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMN\
                 OPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
    let cairo = canvas.to_glib_none();
    let (w, h) = PangoLayout::new(cairo.0, CONFIG.font, string, PangoAttrList::new()).extents();
    canvas.move_to(x_save, y_save);
    ((w / string.len() as i32) as f64, h as f64)
}

fn color(byte: u8) -> f64 {
    byte as f64 / 255.0
}
