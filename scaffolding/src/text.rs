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
use std::cell::Cell;
use std::ops::Range;
use std::rc::Rc;

use notty::cfg;
use notty::datatypes::Color;
use notty::terminal::{CharCell, Styles};

use cairo::Context;
use cairo::glib::translate::ToGlibPtr;

use pangocairo::wrap::{PangoAttribute, PangoAttrList, PangoLayout};

pub struct TextRenderer {
    text: String,
    scroll: Rc<Cell<usize>>,
    fg_color: Vec<(Range<usize>, Color)>,
    bg_color: Vec<(Range<usize>, Color)>,
    opacity: Vec<(Range<usize>, u8)>,
    underline: Vec<Range<usize>>,
    double_underline: Vec<Range<usize>>,
    bold: Vec<Range<usize>>,
    italic: Vec<Range<usize>>,
    strikethrough: Vec<Range<usize>>,
    blink: Vec<Range<usize>>,
}

impl TextRenderer {

    pub fn new(len: usize, scroll: Rc<Cell<usize>>) -> TextRenderer {
        TextRenderer {
            text: String::with_capacity(len),
            scroll: scroll,
            fg_color: Vec::with_capacity(len),
            bg_color: Vec::with_capacity(len),
            opacity: Vec::with_capacity(len),
            underline: Vec::with_capacity(len),
            double_underline: Vec::with_capacity(len),
            bold: Vec::with_capacity(len),
            italic: Vec::with_capacity(len),
            strikethrough: Vec::with_capacity(len),
            blink: Vec::with_capacity(len),
        }
    }

    pub fn draw<'a, Cells>(&mut self,
                           cells: Cells,
                           cursor_pos: usize,
                           cursor_style: Styles,
                           canvas: &Context,
                           width: usize,
                           offset: usize)
    where Cells: Iterator<Item=&'a CharCell> {

        // Line positioning
        canvas.translate(0.0, 0.0);

        // Set text color
        let Color(r,g,b) = cfg::DEFAULT_FG;
        let cast = |x| x as f64 / 255.0;
        canvas.set_source_rgb(cast(r), cast(g), cast(b));

        // Determine offset
        let offset = offset.saturating_sub(self.scroll.get() * width as usize);
        
        // Create the styles and text string
        for (n, cell) in cells.skip(offset).enumerate() {
            if n != 0 && n % (width as usize) == 0 {
                self.text.push('\n');
                self.extend_style();
            }
            let lower = self.text.len();
            match *cell {
                CharCell::Empty(_)              => self.text.push(' '),
                CharCell::Char(ch, _)           => self.text.push(ch),
                CharCell::Grapheme(ref s, _)    => self.text.push_str(&s[..]),
                CharCell::Image { .. }          => self.text.push('X'),
                CharCell::Extension(..)         => self.text.push('X'),
            }
            let range = lower..self.text.len();
            self.add_style(&range, cell.style());
            if n == cursor_pos {
                self.cursor_style(&range, &cursor_style);
            }
        }


        // Draw the text
        let cairo = canvas.to_glib_none();
        PangoLayout::new(cairo.0, cfg::FONT, &self.text, self.pango_attrs()).show(cairo.0);

        // Clear self.
        self.clear();

    }

    fn clear(&mut self) {
        self.text.clear();
        self.fg_color.clear();
        self.bg_color.clear();
        self.opacity.clear();
        self.underline.clear();
        self.double_underline.clear();
        self.bold.clear();
        self.italic.clear();
        self.strikethrough.clear();
        self.blink.clear();
    }

    fn extend_style(&mut self) {
        extend_field(&mut self.bg_color);
        extend_field(&mut self.fg_color);
        extend_field(&mut self.opacity);
        extend_bool(&mut self.underline);
        extend_bool(&mut self.double_underline);
        extend_bool(&mut self.bold);
        extend_bool(&mut self.italic);
        extend_bool(&mut self.strikethrough);
        extend_bool(&mut self.blink);
    }

    fn add_style(&mut self, range: &Range<usize>, style: &Styles) {
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

    fn cursor_style(&mut self, range: &Range<usize>, style: &Styles) {
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

fn extend_bool(ranges: &mut Vec<Range<usize>>) {
    if let Some(last_range) = ranges.last_mut() {
        last_range.end += 1;
    }
}

fn extend_field<T>(ranges: &mut Vec<(Range<usize>, T)>) {
    if let Some(&mut (ref mut last_range, _)) = ranges.last_mut() {
        last_range.end += 1;
    }
}
