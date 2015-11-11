use std::iter::FromIterator;
use std::mem;

use pango::ffi;

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
