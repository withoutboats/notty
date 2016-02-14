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
extern crate gdk;
extern crate gtk;

extern crate tty;
extern crate notty;
extern crate notty_cairo;

use std::cell::{Cell, RefCell};
use std::env;
use std::io::BufReader;
use std::sync::mpsc;
use std::rc::Rc;
use std::thread;

use gtk::{WidgetTrait, WidgetSignals, ContainerTrait};

use notty::{Output, Command, KeyPress, KeyRelease};
use notty::terminal::Terminal;
use notty_cairo::Renderer;

mod commands;
mod key;

use commands::CommandApplicator;
use key::FromEvent;

static mut X_PIXELS: Option<u32> = None;
static mut Y_PIXELS: Option<u32> = None;

static COLS: u32 = 80;
static ROWS: u32 = 25;

fn main() {

    // Set up window and drawing canvas.
    gtk::init().unwrap();
    let window = gtk::Window::new(gtk::WindowType::Toplevel).unwrap();
    let canvas = Rc::new(gtk::DrawingArea::new().unwrap());
    window.add(&*canvas);

    // Set the TERM variable and establish a TTY connection
    env::set_var("TERM", "notty");
    let (tty_r, tty_w) = tty::pty("sh", COLS as u16, ROWS as u16);

    // Handler program output (tty -> screen) on separate thread.
    let (tx_out, rx) = mpsc::channel();
    let (tx_key_press, tx_key_release) = (tx_out.clone(), tx_out.clone());
    thread::spawn(move || {
        let output = Output::new(BufReader::new(tty_r));
        for cmd in output {
            tx_out.send(cmd.unwrap()).unwrap();
        }
    });

    // Create the scroll position tracker.
    let scroll   = Rc::new(Cell::new(0));
    let (scroll2, scroll3) = (scroll.clone(), scroll.clone());

    // Set up logical terminal and renderer.
    let terminal        = Rc::new(RefCell::new(Terminal::new(COLS, ROWS, tty_w)));

    // Process screen logic every 125 milliseconds.
    let cmd = CommandApplicator::new(rx, terminal.clone(), canvas.clone());
    gdk::glib::timeout_add(125, move || cmd.apply());

    // Connect signal to draw on canvas.
    canvas.connect_draw(move |_, canvas| {
        let mut terminal = terminal.borrow_mut();
        let skip = terminal.grid_height.saturating_sub(terminal.height + scroll.get() as u32);
        let renderer = Renderer::new(&canvas, skip);
        if let (Some(x_pix), Some(y_pix)) = unsafe {(X_PIXELS.take(), Y_PIXELS.take())} {
            renderer.reset_dimensions(&mut terminal, x_pix, y_pix);
        }
        renderer.draw(&terminal, &canvas);
        gtk::signal::Inhibit(false)
    });

    // Connect signal for changing window size.
    canvas.connect_configure_event(move |canvas, config| {
        unsafe {
            X_PIXELS = Some(config.width as u32);
            Y_PIXELS = Some(config.height as u32);
        }
        canvas.queue_draw();
        gtk::signal::Inhibit(false)
    });

    // Connect signal to receive key presses.
    window.connect_key_press_event(move |window, event| {
        if let Some(cmd) = KeyPress::from_event(event, &*scroll2) {
            tx_key_press.send(cmd).unwrap();
        } else { window.queue_draw(); }
        gtk::signal::Inhibit(false)
    });

    // Connect signal to receive key releases.
    window.connect_key_release_event(move |window, event| {
        if let Some(cmd) = KeyRelease::from_event(event, &*scroll3) {
            tx_key_release.send(cmd).unwrap();
        } else { window.queue_draw(); }
        gtk::signal::Inhibit(false)
    });

    // Show the window and run the GTK event loop.
    window.show_all();
    gtk::main();

}
