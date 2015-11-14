use std::fmt;

use datatypes::Key;

use self::Tooltip::*;

pub enum Tooltip {
    Basic(String),
    Menu {
        options: Vec<String>,
        position: Option<usize>,
    }
}

impl Tooltip {
    pub fn interact(&mut self, key: &Key) -> Result<usize, bool> {
        match self {
            &mut Menu { ref mut position, .. }   => match (key, position.take()) {
                (&Key::Down, None)      => {
                    *position = Some(0);
                    Err(false)
                }
                (&Key::Down, Some(n))   => {
                    *position = Some(n + 1);
                    Err(false)
                }
                (&Key::Up, Some(0))     => Err(false),
                (&Key::Up, Some(n))     => {
                    *position = Some(n - 1);
                    Err(false)
                }
                (&Key::Enter, Some(n))  => Ok(n),
                _   => Err(true)
            },
            _   => Err(true)
        }
    }
}

impl fmt::Display for Tooltip {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Basic(ref s)                => f.write_str(s),
            Menu { ref options, .. }    => f.write_str(&options.join("\n")),
        }
    }
}
