extern crate gdk;
extern crate gtk;
extern crate pangocairo;
extern crate cairo;

extern crate tty;
extern crate natty;

use std::cell::{Cell, RefCell};
use std::env;
use std::io::BufReader;
use std::sync::mpsc;
use std::rc::Rc;
use std::thread;

use natty::{Input, Output, Screen, Command};
use gtk::{WidgetTrait, WidgetSignals, ContainerTrait};

mod key;
mod text;
mod screen;

use screen::ScreenRenderer;

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
    env::set_var("TERM", "natty");
    let (tty_r, tty_w, handle) = tty::pty("sh", COLS as u16, ROWS as u16);

    // Handle user input (keys -> tty) on separate thread.
    let (tx_in, rx_in) = mpsc::channel();
    thread::spawn(move || Input::new(tty_w).run(rx_in).unwrap());

    // Handler program output (tty -> screen) on separate thread.
    let (tx_out, rx_out) = mpsc::channel();
    thread::spawn(move || Output::new(BufReader::new(tty_r)).run(tx_out).unwrap());

    // Create the scroll position tracker.
    let scroll   = Rc::new(Cell::new(0));
    let (scroll2, scroll3) = (scroll.clone(), scroll.clone());

    // Set up logical screen and renderer.
    let screen   = Rc::new(RefCell::new(Screen::new(COLS, ROWS)));
    let renderer = ScreenRenderer::new(screen.clone(), scroll.clone(), handle);

    //Create clones of the input channel.
    let (tx_in2, tx_in3) = (tx_in.clone(), tx_in.clone());

    // Process screen logic every 100 miliseconds.
    let canvas2 = canvas.clone();
    gdk::glib::timeout_add(100, move || {
        use std::sync::mpsc::TryRecvError::*;

        let mut screen = screen.borrow_mut();
        let mut redraw = false;
        loop {
            match rx_out.try_recv() {
                Ok(cmd)             => {
                    redraw = true;
                    cmd.apply(&mut screen, &tx_in);
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
    window.connect_key_press_event(move |window, key_press| {
        if let Some(k) = key::translate(key_press, &*scroll2) {
            tx_in2.send(k).unwrap();
        } else { window.queue_draw(); }
        gtk::signal::Inhibit(false)
    });

    // Connect signal to receive key releases.
    window.connect_key_release_event(move |window, key_release| {
        if let Some(k) = key::translate(key_release, &*scroll3) {
            tx_in3.send(k).unwrap();
        } else { window.queue_draw(); }
        gtk::signal::Inhibit(false)
    });

    // Show the window and run the GTK event loop.
    window.show_all();
    gtk::main();

}
