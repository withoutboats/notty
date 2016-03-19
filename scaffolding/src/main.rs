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
extern crate cairo;
extern crate gdk;
extern crate gtk;

extern crate tty;
extern crate notty;
extern crate notty_cairo;

use std::cell::RefCell;
use std::env;
use std::io::BufReader;
use std::process;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::rc::Rc;
use std::thread;

use gtk::{WindowTrait, WidgetTrait, WidgetSignals, ContainerTrait};

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

    // Handle program output (tty -> screen) on separate thread.
    let (tx_out, rx) = mpsc::channel();
    let (tx_key_press, tx_key_release) = (tx_out.clone(), tx_out.clone());

    let pty_open = Arc::new(AtomicBool::new(true));
    let pty_open_checker = pty_open.clone();
    thread::spawn(move || {
        let mut output = Output::new(BufReader::new(tty_r));
        while let Some(Ok(cmd)) = output.next() {
            tx_out.send(cmd).unwrap();
        }
        pty_open.store(false, Ordering::SeqCst);
    });

    // Quit GTK main loop if the (tty -> screen) output handler thread indicates
    // pty is no longer open.
    gdk::glib::timeout_add(50, move || {
        if pty_open_checker.load(Ordering::SeqCst) { gdk::glib::Continue(true) }
        else { exit() }
    });

    // Set up logical terminal and renderer.
    let terminal = Rc::new(RefCell::new(Terminal::new(COLS, ROWS, tty_w)));
    let renderer = RefCell::new(Renderer::new());

    // Process screen logic every 25 milliseconds.
    let cmd = CommandApplicator::new(rx, terminal.clone(), canvas.clone());
    gdk::glib::timeout_add(25, move || cmd.apply());

    // Connect signal to draw on canvas.
    canvas.connect_draw(move |_, canvas| {
        let mut terminal = terminal.borrow_mut();
        if let (Some(x_pix), Some(y_pix)) = unsafe {(X_PIXELS.take(), Y_PIXELS.take())} {
            renderer.borrow_mut().reset_dimensions(&canvas, &mut terminal, x_pix, y_pix);
        }
        renderer.borrow_mut().draw(&terminal, &canvas);
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
        if let Some(cmd) = KeyPress::from_event(event) {
            tx_key_press.send(cmd).unwrap();
        } else { window.queue_draw(); }
        gtk::signal::Inhibit(false)
    });

    // Connect signal to receive key releases.
    window.connect_key_release_event(move |window, event| {
        if let Some(cmd) = KeyRelease::from_event(event) {
            tx_key_release.send(cmd).unwrap();
        } else { window.queue_draw(); }
        gtk::signal::Inhibit(false)
    });

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        gtk::signal::Inhibit(false)
    });

    // Show the window and run the GTK event loop.
    window.set_default_size(800, 800);
    window.show_all();
    gtk::main();
}

pub fn exit<T>() -> T {
    gtk::main_quit();
    process::exit(0)
}
