use unicode_width::*;

use command::*;
use datatypes::Key;
use datatypes::Key::*;
use datatypes::Area::*;
use datatypes::Movement::*;
use datatypes::Direction::*;

pub fn encode(key: Key, lerase: char, lnext: char, werase: char) -> Option<Box<Command>> {
    match key {
        Char(c) if c == lerase  => wrap(CommandSeries(vec![
            Box::new(Erase::new(CursorTo(ToEdge(Left)))) as Box<Command>,
            Box::new(Move::new(ToEdge(Left))) as Box<Command>,
        ])),
        Char(c) if c == lnext   => unimplemented!(),
        Char(c) if c == werase  => unimplemented!(),
        Char(c) if c.width().is_some() => wrap(Put::new_char(c)),
        LeftArrow   => wrap(Move::new(To(Left, 1, false))),
        RightArrow  => wrap(Move::new(To(Right, 1, false))),
        Enter       => wrap(Move::new(NextLine(1))),
        Backspace   => wrap(CommandSeries(vec![
            Box::new(Move::new(To(Left, 1, false))) as Box<Command>,
            Box::new(RemoveChars::new(1)) as Box<Command>,
        ])),
        Delete      => wrap(RemoveChars::new(1)),
        Home        => wrap(Move::new(ToEdge(Left))),
        _           => None
    }
}

fn wrap<T: Command>(cmd: T) -> Option<Box<Command>> {
    Some(Box::new(cmd) as Box<Command>)
}
