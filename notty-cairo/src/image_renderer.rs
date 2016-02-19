use std::mem;
use std::ptr;

use cairo;

use gdk::Pixbuf;
use gdk::cairo_interaction::ContextExt;
use gdk::glib::translate::FromGlibPtr;
use gio;
use pixbuf;
use libc;

use notty::terminal::Styles;

pub struct ImageRenderer {
    pixbuf: Pixbuf,
    x_pos: f64,
    y_pos: f64,
}

impl ImageRenderer {
    pub fn new(data: &[u8], style: Styles, x: f64, y: f64) -> ImageRenderer {
        fn pixbuf_from_data(data: &[u8]) -> Option<Pixbuf> {
            let null = ptr::null_mut();
            unsafe {
                let (data, len) = (mem::transmute(data.as_ptr()), data.len() as libc::ssize_t);
                let stream = gio::g_memory_input_stream_new_from_data(data, len, None);
                let pixbuf = pixbuf::gdk_pixbuf_new_from_stream(stream, null, null as *mut _);
                if pixbuf != null as *mut _ { Some(Pixbuf::from_glib_full(pixbuf)) }
                else { None }
            }
        }
        fn empty_pixbuf() -> Pixbuf {
            unsafe { Pixbuf::new(0, false, 0, 1, 1).expect("Could not create empty Pixbuf.") }
        }

        ImageRenderer {
            pixbuf: pixbuf_from_data(data).unwrap_or_else(empty_pixbuf),
            x_pos: x,
            y_pos: y,
        }
    }

    pub fn draw(&self, canvas: &cairo::Context) {
        canvas.set_source_pixbuf(&self.pixbuf, self.x_pos, self.y_pos);
        canvas.paint();
    }
}
