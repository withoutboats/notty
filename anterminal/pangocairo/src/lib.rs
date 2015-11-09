extern crate gobject_sys;
extern crate pango;
extern crate cairo;

pub mod ffi;

pub mod wrap {
    mod attr;
    mod attr_list;
    mod layout;

    pub use self::attr::PangoAttribute;
    pub use self::attr_list::PangoAttrList;
    pub use self::layout::PangoLayout;
}
