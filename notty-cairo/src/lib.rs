#![feature(arc_counts)]
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

use std::collections::HashMap;
use std::sync::Arc;

use gdk::glib::translate::ToGlibPtr;

use itertools::Itertools;

use notty::datatypes::{Coords, Color};
use notty::cfg::Config;
use notty::terminal::{CharData, Terminal, ImageData};

use pangocairo::wrap::{PangoLayout, PangoAttrList};

use self::image_renderer::ImageRenderer;
use self::text_renderer::TextRenderer;

pub struct Renderer {
    images: HashMap<Arc<ImageData>, ImageRenderer>,
    char_d: Option<(f64, f64)>,
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer {
            images: HashMap::new(),
            char_d: None
        }
    }

    pub fn reset_dimensions(&mut self, canvas: &cairo::Context, terminal: &mut Terminal,
                            pix_w: u32, pix_h: u32) {
        let (char_w, char_h) = self.char_d.unwrap_or_else(|| {
            let char_d = self.char_dimensions(canvas, &terminal.config);
            self.char_d = Some(char_d);
            char_d
        });
        let width = pix_w / (char_w as u32);
        let height = pix_h / (char_h as u32);
        terminal.set_winsize(Some(width), Some(height)).unwrap_or_else(|e| panic!("{}", e));
    }

    pub fn draw(&mut self, terminal: &Terminal, canvas: &cairo::Context) {
        if self.char_d.is_none() {
            self.char_d = Some(self.char_dimensions(canvas, &terminal.config));
        }
        let Color(r,g,b) = terminal.config.bg_color;
        canvas.set_source_rgb(color(r), color(g), color(b));
        canvas.paint();

        let col_n = terminal.area().width() as usize;
        let rows = terminal.cells().chunks_lazy(col_n);

        // Remove dead images from the cache.
        for key in self.images.keys().filter(|k| Arc::strong_count(k) == 1).cloned().collect::<Vec<_>>() {
            self.images.remove(&key);
        }

        for (y_pos, row) in rows.into_iter().enumerate() {
            let y_pix = self.y_pixels(y_pos as u32);
            let mut text = TextRenderer::new(0.0, y_pix);
            for (x_pos, cell) in row.enumerate() {
                let style = cell.styles;
                if (Coords { x: x_pos as u32, y: y_pos as u32 } == terminal.cursor_position()) {
                    let cursor_style = terminal.cursor_styles();
                    match cell.content {
                        CharData::Empty             => text.push_cursor(' ', style, cursor_style),
                        CharData::Char(ch)          => text.push_cursor(ch, style, cursor_style),
                        CharData::Grapheme(ref s)   => text.push_str_cursor(s, style, cursor_style),
                        CharData::Extension(_)      => unreachable!(),
                        CharData::Image { .. }      => continue,
                    }
                    continue;
                }
                match cell.content {
                    CharData::Empty             => text.push(' ', style),
                    CharData::Char(ch)          => text.push(ch, style),
                    CharData::Grapheme(ref s)   => text.push_str(s, style),
                    CharData::Extension(_)      => { }
                    CharData::Image { ref data, ref pos, ref width, ref height, .. } => {
                        let x_pix = self.x_pixels(x_pos as u32);
                        if (x_pos + *width as usize) < col_n {
                            text.draw(canvas, &terminal.config.font, terminal.config.bg_color);
                            text = TextRenderer::new(x_pix, y_pix);
                        }
                        if let Some(image) = self.images.get(data) {
                            image.draw(canvas);
                            continue;
                        }
                        let w_pix = self.x_pixels(*width);
                        let h_pix = self.y_pixels(*height);
                        let img = ImageRenderer::new(&data.data, x_pix, y_pix, w_pix, h_pix,
                                                     *pos);
                        img.draw(canvas);
                        self.images.insert(data.clone(), img);
                    }
                }
            }
            text.draw(canvas, &terminal.config.font, terminal.config.bg_color);
        }
    }

    fn char_dimensions(&self, canvas: &cairo::Context, config: &Config) -> (f64, f64) {
        //save the canvas position
        let (x_save, y_save) = canvas.get_current_point();
        let string = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMN\
                      OPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
        let cairo = canvas.to_glib_none();
        let (w, h) = PangoLayout::new(cairo.0, &config.font, string,
                                      PangoAttrList::new()).extents();
        canvas.move_to(x_save, y_save);
        ((w / string.len() as i32) as f64, h as f64)
    }

    fn x_pixels(&self, x: u32) -> f64 {
        self.char_d.unwrap().0 * (x as f64)
    }

    fn y_pixels(&self, y: u32) -> f64 {
        self.char_d.unwrap().1 * (y as f64)
    }
}

fn color(byte: u8) -> f64 {
    byte as f64 / 255.0
}
