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
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use cairo::Context;
use notty::cfg;
use notty::datatypes::Color;
use notty::terminal::Terminal;

use text::TextRenderer;
use tty::Handle;

use {X_PIXELS, Y_PIXELS};

pub struct ScreenRenderer {
    logic: Rc<RefCell<Terminal>>,
    text: RefCell<TextRenderer>,
    red: f64,
    green: f64,
    blue: f64,
    tty: Arc<Handle>,
}

impl ScreenRenderer {

    pub fn new(logic: Rc<RefCell<Terminal>>, scroll: Rc<Cell<usize>>, tty: Arc<Handle>)
            -> ScreenRenderer {

        let len = logic.borrow().width as usize * logic.borrow().height as usize;
        let Color(r,g,b) = cfg::DEFAULT_BG;
        ScreenRenderer {
            logic: logic,
            text: RefCell::new(TextRenderer::new(len, scroll)),
            red: r as f64 / 255.0,
            green: g as f64 / 255.0,
            blue: b as f64 / 255.0,
            tty: tty,
        }
        
    }

    pub fn draw(&self, canvas: Context) {

        // Reset terminal dimensions.
        if let (Some(x_pixels), Some(y_pixels)) = unsafe { (X_PIXELS.take(), Y_PIXELS.take()) } {
            let mut logic = self.logic.borrow_mut();
            let f_extents = canvas.font_extents();
            let cs = x_pixels / (f_extents.max_x_advance as u32 + 2);
            let rs = y_pixels / (f_extents.height as u32 + 4);
            logic.set_visible_width(cs);
            logic.set_visible_height(rs);
            self.tty.set_winsize(cs as u16, rs as u16).unwrap();
        }

        // Paint background.
        canvas.set_source_rgb(self.red, self.blue, self.green);
        canvas.paint();

        // Render the text
        let logic = self.logic.borrow();
        let offset = (logic.grid_height.saturating_sub(logic.height)) * logic.grid_width;
        let coords = logic.cursor_position();
        if let Some(info) = logic.tooltip_at(coords) {
            println!("{}", info);
        }
        let cursor_pos = ((coords.y * logic.grid_width) + coords.x).saturating_sub(offset);
        self.text.borrow_mut().draw(logic.into_iter(),
                                    cursor_pos as usize,
                                    logic.cursor_styles(),
                                    &canvas,
                                    logic.grid_width as usize,
                                    offset as usize);

    }

}
