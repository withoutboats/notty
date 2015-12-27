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
use cairo::ffi as cairo;
use pango::ffi as pango;

#[link(name = "pangocairo-1.0")]
extern {
    pub fn pango_cairo_create_layout(cr: *mut cairo::cairo_t) -> *mut pango::PangoLayout;
    pub fn pango_cairo_show_layout(cr: *mut cairo::cairo_t, layout: *mut pango::PangoLayout);
}
