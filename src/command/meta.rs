use std::cell::RefCell;

use command::prelude::*;
use datatypes::InputMode;

#[derive(Copy, Clone)]
pub struct PushBuffer(pub bool);

impl Command for PushBuffer {
    fn apply(&self, screen: &mut Screen, _: &Sender<InputEvent>) {
        screen.push_buffer(false, self.0);
    }
    fn repr(&self) -> String {
        match self.0 {
            true    => String::from("PUSH BUFFER SCROLLING"),
            false   => String::from("PUSH BUFFER STATIC"),
        }
    }
}

#[derive(Copy, Clone)]
pub struct PopBuffer;

impl Command for PopBuffer {
    fn apply(&self, screen: &mut Screen, _: &Sender<InputEvent>) {
        screen.pop_buffer();
    }
    fn repr(&self) -> String {
        String::from("POP BUFFER")
    }
}

pub struct SetTitle(pub RefCell<Option<String>>);

impl Command for SetTitle {
    fn apply(&self, screen: &mut Screen, _: &Sender<InputEvent>) {
        if let Some(title) = self.0.borrow_mut().take() {
            screen.set_title(title);
        }
    }
    fn repr(&self) -> String {
        String::from("SET TITLE")
    }
}

#[derive(Copy, Clone)]
pub struct SetInputMode(pub InputMode);

impl Command for SetInputMode {
    fn apply(&self, _: &mut Screen, input: &Sender<InputEvent>) {
        input.send(InputEvent::Mode(self.0));
    }
    fn repr(&self) -> String {
        match self.0 {
            InputMode::Ansi         => String::from("SET INPUTMODE ANSI"),
            InputMode::Application  => String::from("SET INPUTMODE APPLICATION"),
            InputMode::Extended     => String::from("SET INPUTMODE EXTENDED"),
        }
    }
}

#[derive(Default, Copy, Clone)]
pub struct Bell;

impl Command for Bell {
    fn apply(&self, screen: &mut Screen, _: &Sender<InputEvent>) {
        screen.bell();
    }
    fn repr(&self) -> String {
        String::from("BELL")
    }
}
