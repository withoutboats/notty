extern crate gdk;
extern crate gtk;
extern crate pangocairo;
extern crate cairo;

extern crate tty;
extern crate notty;

use std::cell::{Cell, RefCell};
use std::env;
use std::io::BufReader;
use std::sync::mpsc;
use std::rc::Rc;
use std::thread;

use notty::{Output, Command, KeyPress, KeyRelease};
use notty::terminal::Terminal;
use gtk::{WidgetTrait, WidgetSignals, ContainerTrait};

mod key;
mod text;
mod screen;

use screen::ScreenRenderer;
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
    let (tty_r, tty_w, handle) = tty::pty("sh", COLS as u16, ROWS as u16);

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
    let terminal   = Rc::new(RefCell::new(Terminal::new(COLS, ROWS, tty_w)));
    let renderer = ScreenRenderer::new(terminal.clone(), scroll.clone(), handle);

    // Process screen logic every 50 miliseconds.
    let canvas2 = canvas.clone();
    gdk::glib::timeout_add(50, move || {
        use std::sync::mpsc::TryRecvError::*;

        let mut terminal = terminal.borrow_mut();
        let mut redraw = false;
        loop {
            match rx.try_recv() {
                Ok(cmd)             => {
                    redraw = true;
                    cmd.apply(&mut terminal).unwrap();
                }
                Err(Disconnected)   => {
                    gtk::main_quit();
                    panic!();
                }
                Err(Empty)          => break,
            }
        }
        if redraw { canvas2.queue_draw(); }
        gdk::glib::Continue(true)
    });

    // Connect signal to draw on canvas.
    canvas.connect_draw(move |_, canvas| {
        renderer.draw(canvas);
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
