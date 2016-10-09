#![feature(arc_counts)]
extern crate cairo;
extern crate gdk;
extern crate gdk_pixbuf as pixbuf;
extern crate gdk_pixbuf_sys as pixbuf_sys;
extern crate gio_sys as gio;
extern crate glib;
extern crate itertools;
extern crate libc;
extern crate notty;
extern crate pangocairo;

mod cfg;
mod image_renderer;
mod text_renderer;

use std::collections::HashMap;
use std::sync::Arc;

use glib::translate::ToGlibPtr;

use itertools::Itertools;

use notty::datatypes::Coords;
use notty::terminal::{CellData, CharGrid, Fill, Terminal, ImageData, Styleable, Resizeable};

use pangocairo::wrap::{PangoLayout, PangoAttrList};

use self::cfg::gtk_color;
use self::image_renderer::ImageRenderer;
use self::text_renderer::TextRenderer;

pub use self::cfg::{Config, TrueColor, PALETTE};

pub struct Renderer {
    images: HashMap<Arc<ImageData>, ImageRenderer>,
    char_d: Option<(f64, f64)>,
    cfg: Config,
}

impl Renderer {
    pub fn new(cfg: Config) -> Renderer {
        Renderer {
            images: HashMap::new(),
            char_d: None,
            cfg: cfg,
        }
    }

    pub fn reset_dimensions(&mut self, canvas: &cairo::Context, terminal: &mut Terminal,
                            pix_w: u32, pix_h: u32) {
        let (char_w, char_h) = self.char_d.unwrap_or_else(|| {
            let char_d = self.char_dimensions(canvas);
            self.char_d = Some(char_d);
            char_d
        });
        let width = pix_w / (char_w as u32);
        let height = pix_h / (char_h as u32);
        terminal.set_winsize(Some(width), Some(height)).unwrap_or_else(|e| panic!("{}", e));
    }

    pub fn draw_grid(&mut self, grid: &CharGrid, canvas: &cairo::Context) {
        let col_n = grid.dims().0 as usize;
        let rows = grid.cells().chunks(col_n);

        // Remove dead images from the cache.
        for key in self.images.keys().filter(|k| Arc::strong_count(k) == 1).cloned().collect::<Vec<_>>() {
            self.images.remove(&key);
        }

        for (y_pos, row) in rows.into_iter().enumerate() {
            let y_pix = self.y_pixels(y_pos as u32);
            let mut text = TextRenderer::new(&self.cfg, 0.0, y_pix);
            for (x_pos, cell) in row.enumerate() {
                let style = *cell.styles();
                if (Coords { x: x_pos as u32, y: y_pos as u32 } == grid.cursor().position()) {
                    let cursor_style = *grid.cursor().styles();
                    match *cell.content() {
                        CellData::Empty             => text.push_cursor(' ', style, cursor_style),
                        CellData::Char(ch)          => text.push_cursor(ch, style, cursor_style),
                        CellData::Grapheme(ref s)   => text.push_str_cursor(s, style, cursor_style),
                        CellData::Extension(_)      => unreachable!(),
                        CellData::Image { .. }      => continue,
                    }
                    continue;
                }
                match *cell.content() {
                    CellData::Empty             => text.push(' ', style),
                    CellData::Char(ch)          => text.push(ch, style),
                    CellData::Grapheme(ref s)   => text.push_str(s, style),
                    CellData::Extension(_)      => { }
                    CellData::Image { ref data, ref pos, ref width, ref height, .. } => {
                        let x_pix = self.x_pixels(x_pos as u32);
                        if (x_pos + *width as usize) < col_n {
                            text.draw(canvas);
                            text = TextRenderer::new(&self.cfg, x_pix, y_pix);
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
            text.draw(canvas);
        }
    }

    pub fn draw(&mut self, terminal: &Terminal, canvas: &cairo::Context) {

        if self.char_d.is_none() { self.char_d = Some(self.char_dimensions(canvas)); }
        let (r, g, b) = gtk_color(self.cfg.bg_color);
        canvas.set_source_rgb(r, g, b);
        canvas.paint();

        for panel in terminal.panels() {
            match *panel {
                Fill::Grid(ref grid)    => self.draw_grid(grid, canvas),
                _                       => unimplemented!()
            }
        }
    }

    fn char_dimensions(&self, canvas: &cairo::Context) -> (f64, f64) {
        //save the canvas position
        let (x_save, y_save) = canvas.get_current_point();
        let string = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMN\
                      OPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
        let cairo = canvas.to_glib_none();
        let (w, h) = PangoLayout::new(cairo.0, &self.cfg.font, string, PangoAttrList::new()).extents();
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
