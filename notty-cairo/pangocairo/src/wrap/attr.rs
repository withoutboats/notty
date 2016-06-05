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
use std::ops::Range;

use pango;
use pango_sys as ffi;

pub struct PangoAttribute(*mut ffi::PangoAttribute);

impl PangoAttribute {

    pub fn fg_color(range: &Range<usize>, (r,g,b): (u8, u8, u8)) -> PangoAttribute {
        let (r, g, b) = ((r as u16) << 8, (g as u16) << 8, (b as u16) << 8);
        let attr = unsafe { ffi::pango_attr_foreground_new(r, g, b) };
        PangoAttribute(attr).with_range(range)
    }

    pub fn bg_color(range: &Range<usize>, (r,g,b): (u8, u8, u8)) -> PangoAttribute {
        let (r, g, b) = ((r as u16) << 8, (g as u16) << 8, (b as u16) << 8);
        let attr = unsafe { ffi::pango_attr_background_new(r, g, b) };
        PangoAttribute(attr).with_range(range)
    }

    pub fn underline(range: &Range<usize>) -> PangoAttribute {
        let attr = unsafe { ffi::pango_attr_underline_new(ffi::PangoUnderline::Single) };
        PangoAttribute(attr).with_range(range)
    }

    pub fn double_underline(range: &Range<usize>) -> PangoAttribute {
        let attr = unsafe { ffi::pango_attr_underline_new(ffi::PangoUnderline::Double) };
        PangoAttribute(attr).with_range(range)
    }

    pub fn bold(range: &Range<usize>) -> PangoAttribute {
        let attr = unsafe { ffi::pango_attr_weight_new(pango::Weight::Bold) };
        PangoAttribute(attr).with_range(range)
    }

    pub fn italic(range: &Range<usize>) -> PangoAttribute {
        let attr = unsafe { ffi::pango_attr_style_new(pango::Style::Italic) };
        PangoAttribute(attr).with_range(range)
    }

    pub fn strikethrough(range: &Range<usize>) -> PangoAttribute {
        let attr = unsafe { ffi::pango_attr_strikethrough_new(!0) };
        PangoAttribute(attr).with_range(range)
    }

    fn with_range(self, range: &Range<usize>) -> PangoAttribute {
        unsafe {
            (*self.0).start_index = range.start as u32;
            (*self.0).end_index = range.end as u32;
        }
        self
    }

    pub unsafe fn raw(&self) -> *mut ffi::PangoAttribute {
        self.0
    }

}

impl Drop for PangoAttribute {
    fn drop(&mut self) {
        unsafe { ffi::pango_attribute_destroy(self.0); }
    }
}
