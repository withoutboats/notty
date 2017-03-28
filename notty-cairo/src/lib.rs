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
extern crate vec_map;

mod cfg;
mod text_renderer;

use std::cell::RefCell;
use std::mem;
use std::ptr;

use gdk::prelude::ContextExt;

use glib::translate::{FromGlibPtr, ToGlibPtr};

use itertools::Itertools;

use notty::datatypes::{Coords, MediaPosition};
use notty::terminal::{CellData, CharGrid, Fill, Terminal, Styleable, Resizeable};

use pangocairo::wrap::{PangoLayout, PangoAttrList};

use pixbuf::{InterpType, Pixbuf};

use vec_map::VecMap;

use self::cfg::gtk_color;
use self::text_renderer::TextRenderer;

pub use self::cfg::{Config, TrueColor, PALETTE};

pub struct Renderer {
    image_cache: RefCell<VecMap<Pixbuf>>,
    char_d: Option<(f64, f64)>,
    cfg: Config,
}

impl Renderer {
    pub fn new(cfg: Config) -> Renderer {
        Renderer {
            image_cache: RefCell::new(VecMap::new()),
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
                        CellData::Image(_)          => continue,
                    }
                    continue;
                }
                match *cell.content() {
                    CellData::Empty             => text.push(' ', style),
                    CellData::Char(ch)          => text.push(ch, style),
                    CellData::Grapheme(ref s)   => text.push_str(s, style),
                    CellData::Extension(_)      => { }
                    CellData::Image(ref image)  => {
                        let (width, height) = image.dims();
                        let x_pix = self.x_pixels(x_pos as u32);
                        if (x_pos + width as usize) < col_n {
                            text.draw(canvas);
                            text = TextRenderer::new(&self.cfg, x_pix, y_pix);
                        }
                        let w_pix = self.x_pixels(width);
                        let h_pix = self.y_pixels(height);
                        let tag = image.intern(|data| self.intern_image(&data.data, w_pix, h_pix, image.pos()));
                        let cache = self.image_cache.borrow();
                        let pixbuf = cache.get(tag).expect("image must have been interned");
                        self.draw_image(pixbuf, x_pix, y_pix, canvas);
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

    fn intern_image(&self, data: &[u8], w: f64, h: f64, pos: MediaPosition) -> usize {
        let pixbuf = pixbuf_from_data(data).and_then(|img| {
            match pos {
                MediaPosition::Display(_, _) => unimplemented!(),
                MediaPosition::Fill => unimplemented!(),
                MediaPosition::Fit => unimplemented!(),
                MediaPosition::Stretch => {
                    img.scale_simple(w as i32, h as i32, InterpType::Bilinear).ok()
                }
                MediaPosition::Tile => unimplemented!(),
            }
        }).unwrap_or_else(empty_pixbuf);
        let tag = self.image_cache.borrow().len();
        self.image_cache.borrow_mut().insert(tag, pixbuf);
        tag
    }

    fn draw_image(&self, buf: &Pixbuf, x: f64, y: f64, canvas: &cairo::Context) {
        canvas.set_source_pixbuf(buf, x, y);
        canvas.paint();
    }
}

fn pixbuf_from_data(data: &[u8]) -> Option<Pixbuf> {
    let null = ptr::null_mut();
    unsafe {
        let (data, len) = (mem::transmute(data.as_ptr()), data.len() as libc::ssize_t);
        let stream = gio::g_memory_input_stream_new_from_data(data, len, None);
        let pixbuf = pixbuf_sys::gdk_pixbuf_new_from_stream(stream, null, null as *mut _);
        if pixbuf != null as *mut _ { Some(Pixbuf::from_glib_full(pixbuf)) }
        else { None }
    }
}
fn empty_pixbuf() -> Pixbuf {
    unsafe { Pixbuf::new(0, false, 0, 1, 1).expect("Could not create empty Pixbuf.") }
}
