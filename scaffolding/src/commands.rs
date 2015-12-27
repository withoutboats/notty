use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError::*;

use gdk;
use gtk::{self, WidgetTrait};

use notty::Command;
use notty::terminal::Terminal;

pub struct CommandApplicator {
    rx: Receiver<Box<Command>>,
    terminal: Rc<RefCell<Terminal>>,
    canvas: Rc<gtk::DrawingArea>,
}

impl CommandApplicator {

    pub fn new(rx: Receiver<Box<Command>>,
               terminal: Rc<RefCell<Terminal>>,
               canvas: Rc<gtk::DrawingArea>) -> CommandApplicator {
        CommandApplicator { rx: rx, terminal: terminal, canvas: canvas }
    }

    pub fn apply(&self) -> gdk::glib::Continue {
        let mut terminal = self.terminal.borrow_mut();
        let mut redraw = false;
        loop {
            match self.rx.try_recv() {
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
        if redraw { self.canvas.queue_draw(); }
        gdk::glib::Continue(true)
    }

}

unsafe impl Send for CommandApplicator { }
