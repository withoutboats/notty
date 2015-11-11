use cairo::ffi as cairo;
use pango::ffi as pango;

#[link(name = "pangocairo-1.0")]
extern {
    pub fn pango_cairo_create_layout(cr: *mut cairo::cairo_t) -> *mut pango::PangoLayout;
    pub fn pango_cairo_show_layout(cr: *mut cairo::cairo_t, layout: *mut pango::PangoLayout);
}
