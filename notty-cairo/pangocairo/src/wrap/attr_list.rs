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
use std::iter::FromIterator;
use std::mem;

use pango_sys as ffi;

use super::PangoAttribute;

pub struct PangoAttrList(*mut ffi::PangoAttrList);

impl PangoAttrList {
    pub fn new() -> PangoAttrList {
        PangoAttrList(unsafe { ffi::pango_attr_list_new() })
    }

    pub fn push(&self, attribute: PangoAttribute) {
        unsafe {
            ffi::pango_attr_list_insert(self.0, attribute.raw());
        }
        mem::forget(attribute);
    }

    pub unsafe fn raw(&self) -> *mut ffi::PangoAttrList {
        self.0
    }
}

impl Clone for PangoAttrList {
    fn clone(&self) -> PangoAttrList {
        PangoAttrList(unsafe { ffi::pango_attr_list_ref(self.0) })
    }
}

impl Drop for PangoAttrList {
    fn drop(&mut self) {
        unsafe { ffi::pango_attr_list_unref(self.0); }
    }
}

impl FromIterator<PangoAttribute> for PangoAttrList {
    fn from_iter<T: IntoIterator<Item=PangoAttribute>>(iterator: T) -> PangoAttrList {
        let list = PangoAttrList::new();
        for attr in iterator {
            list.push(attr);
        }
        list
    }
}
