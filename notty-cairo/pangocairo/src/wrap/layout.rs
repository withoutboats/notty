//  notty is a new kind of terminal emulator.
//  Copyright (C) 2015 without boats
//  
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//  
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU Affero General Public License for more details.
//  
//  You should have received a copy of the GNU Affero General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
use std::ffi::CString;

use gobject_sys;
use cairo::ffi as cairo;
use pango::ffi as pango;

use ffi as pangocairo;

use super::PangoAttrList;

pub struct PangoLayout(*mut pango::PangoLayout);

impl PangoLayout {

    pub fn new(cairo: *mut cairo::cairo_t,
               font: &str,
               text: &str,
               attributes: PangoAttrList) -> PangoLayout {
        let font = CString::new(font.as_bytes()).unwrap().as_ptr();
        unsafe {
            let layout = pangocairo::pango_cairo_create_layout(cairo);
            let font = pango::pango_font_description_from_string(font);
            pango::pango_layout_set_font_description(layout, font);
            pango::pango_font_description_free(font);
            pango::pango_layout_set_text(layout,
                                         text.as_bytes().as_ptr() as *const i8,
                                         text.len() as i32);
            pango::pango_layout_set_attributes(layout, attributes.raw());
            PangoLayout(layout)
        }
    }

    pub fn show(&self, cairo: *mut cairo::cairo_t) {
        unsafe {
            pangocairo::pango_cairo_show_layout(cairo, self.raw());
        }
    }

    pub unsafe fn raw(&self) -> *mut pango::PangoLayout {
        self.0
    }

}

impl Clone for PangoLayout {
    fn clone(&self) -> PangoLayout {
        unsafe { PangoLayout(gobject_sys::g_object_ref(self.0 as *mut _) as *mut _) }
    }
}

impl Drop for PangoLayout {
    fn drop(&mut self) {
        unsafe { gobject_sys::g_object_unref(self.0 as *mut _); }
    }
}
