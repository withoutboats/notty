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

use notty::datatypes::Color;
use notty::terminal::Styles;

use cairo;
use cairo::glib::translate::ToGlibPtr;

use pangocairo::wrap::{PangoAttribute, PangoAttrList, PangoLayout};

use color;

pub struct TextRenderer {
    text: String,
    fg_color: Vec<(Range<usize>, Color)>,
    bg_color: Vec<(Range<usize>, Color)>,
    opacity: Vec<(Range<usize>, u8)>,
    underline: Vec<Range<usize>>,
    double_underline: Vec<Range<usize>>,
    bold: Vec<Range<usize>>,
    italic: Vec<Range<usize>>,
    strikethrough: Vec<Range<usize>>,
    blink: Vec<Range<usize>>,
    x_pos: f64,
    y_pos: f64,
}

impl TextRenderer {

    pub fn new(x: f64, y: f64) -> TextRenderer {
        TextRenderer {
            text: String::new(),
            fg_color: Vec::new(),
            bg_color: Vec::new(),
            opacity: Vec::new(),
            underline: Vec::new(),
            double_underline: Vec::new(),
            bold: Vec::new(),
            italic: Vec::new(),
            strikethrough: Vec::new(),
            blink: Vec::new(),
            x_pos: x,
            y_pos: y,
        }
    }

    pub fn push(&mut self, ch: char, styles: Styles) {
        let lower = self.text.len();
        self.text.push(ch);
        let range = lower..self.text.len();
        self.add_style(&range, styles);
    }

    pub fn push_str(&mut self, s: &str, styles: Styles) {
        let lower = self.text.len();
        self.text.push_str(s);
        let range = lower..self.text.len();
        self.add_style(&range, styles);
    }

    pub fn draw(&self, canvas: &cairo::Context, font: &'static str, bg_color: Color) {
        if self.is_blank(bg_color) { return; }

        // Line positioning
        canvas.move_to(self.x_pos, self.y_pos);

        // Set text color
        let Color(r,g,b) = bg_color;
        canvas.set_source_rgb(color(r), color(g), color(b));

        // Draw the text
        let cairo = canvas.to_glib_none();
        PangoLayout::new(cairo.0, font, &self.text, self.pango_attrs()).show(cairo.0);
    }

    fn is_blank(&self, bg_color: Color) -> bool {
        self.text.chars().all(char::is_whitespace)
        && self.bg_color.iter().all(|&(_, color)| color == bg_color)
    }

    fn add_style(&mut self, range: &Range<usize>, style: Styles) {
        if !style.inverted {
            append_field(range.clone(), style.fg_color, &mut self.fg_color);
            append_field(range.clone(), style.bg_color, &mut self.bg_color);
        } else {
            append_field(range.clone(), style.bg_color, &mut self.fg_color);
            append_field(range.clone(), style.fg_color, &mut self.bg_color);
        }
        append_field(range.clone(), style.opacity, &mut self.opacity);
        if style.underline { append_bool(range.clone(), &mut self.underline) }
        if style.double_underline { append_bool(range.clone(), &mut self.double_underline) }
        if style.bold { append_bool(range.clone(), &mut self.bold); }
        if style.italic { append_bool(range.clone(), &mut self.italic) }
        if style.strikethrough { append_bool(range.clone(), &mut self.strikethrough); }
        if style.blink { append_bool(range.clone(), &mut self.blink) }
    }

    fn cursor_style(&mut self, range: &Range<usize>, style: Styles) {
        if let Some(&mut (ref mut prev_range, ref mut color)) = self.bg_color.last_mut() {
            if style.fg_color == *color { return; }
            if prev_range == range {
                *color = style.fg_color;
                return;
            } else {
                prev_range.end = range.start;
            }
        }
        self.bg_color.push((range.clone(), style.fg_color));
    }

    fn pango_attrs(&self) -> PangoAttrList {
        self.fg_color.iter().map(|&(ref range, Color(r,g,b))| {
            PangoAttribute::fg_color(range, (r,g,b))
        }).chain(self.bg_color.iter().map(|&(ref range, Color(r,g,b))| {
            PangoAttribute::bg_color(range, (r,g,b))
        })).chain(self.underline.iter().map(PangoAttribute::underline))
        .chain(self.double_underline.iter().map(PangoAttribute::double_underline))
        .chain(self.bold.iter().map(PangoAttribute::bold))
        .chain(self.italic.iter().map(PangoAttribute::italic))
        .chain(self.strikethrough.iter().map(PangoAttribute::strikethrough))
        .collect()
    }

}

fn append_bool(range: Range<usize>, ranges: &mut Vec<Range<usize>>) {
    if let Some(last_range) = ranges.last_mut() {
        if last_range.end == range.start {
            return last_range.end = range.end;
        }
    }
    ranges.push(range);
}

fn append_field<T>(range: Range<usize>, field: T, ranges: &mut Vec<(Range<usize>, T)>)
where T: PartialEq + Copy {
    if let Some(&mut (ref mut last_range, last_field)) = ranges.last_mut() {
        if last_field == field {
            last_range.end = range.end;
            return;
        }
    }
    ranges.push((range, field));
}
