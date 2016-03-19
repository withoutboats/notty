use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use std::result;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError::*;

use gtk::{self, WidgetTrait};

use notty::Command;
use notty::terminal::Terminal;

pub struct CommandApplicator {
    rx: Receiver<Command>,
    terminal: Rc<RefCell<Terminal>>,
    canvas: Rc<gtk::DrawingArea>,
}

pub enum CommandError {
    Io(io::Error),
    Disconnected,
}

pub type Result<T> = result::Result<T, CommandError>;

impl CommandApplicator {

    pub fn new(rx: Receiver<Command>,
               terminal: Rc<RefCell<Terminal>>,
               canvas: Rc<gtk::DrawingArea>) -> CommandApplicator {
        CommandApplicator { rx: rx, terminal: terminal, canvas: canvas }
    }

    pub fn apply(&self) -> Result<()> {
        let mut terminal = self.terminal.borrow_mut();
        let mut redraw = false;
        loop {
            match self.rx.try_recv() {
                Ok(cmd)             => {
                    match terminal.apply(&cmd) {
                        Err(e) => return Err(CommandError::Io(e)),
                        _ => {},
                    }
                    redraw = true;
                },
                Err(Disconnected)   => {
                    return Err(CommandError::Disconnected);
                },
                Err(Empty)          => break,
            }
        }
        if redraw { self.canvas.queue_draw(); }
        Ok(())
    }

}

unsafe impl Send for CommandApplicator { }
