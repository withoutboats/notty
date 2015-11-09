use datatypes::Coords;
use screen::Styles;

use self::CharCell::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CharCell {
    Empty(Styles),
    Char(char, Styles),
    Grapheme(String, Styles),
    Data(MimeType, Vec<u8>, Styles),
    Extension(Coords, Styles),
}

impl CharCell {

    pub fn character(ch: char, style: Styles) -> CharCell {
        Char(ch, style)
    }

    pub fn grapheme(grapheme: String, style: Styles) -> CharCell {
       Grapheme(grapheme, style)
    }

    pub fn extend_by(&mut self, ext: char) -> bool {
        match *self {
            Char(c, style)          => {
                let mut string = c.to_string();
                string.push(ext);
                *self = Grapheme(string, style);
                true
            }
            Grapheme(ref mut s, _)  => {
                s.push(ext);
                true
            }
            _                       => {
                false
            }
        }
    }

    pub fn style(&self) -> &Styles {
        match *self {
            Char(_, ref style)
                | Grapheme(_, ref style)
                | Empty(ref style)
                | Data(_, _, ref style)
                | Extension(_, ref style)
                => style
        }
    }

    pub fn style_mut(&mut self) -> &mut Styles {
        match *self {
            Char(_, ref mut style)
                | Grapheme(_, ref mut style)
                | Empty(ref mut style)
                | Data(_, _, ref mut style)
                | Extension(_, ref mut style)
                => style
        }
    }

    pub fn empty(&mut self) {
        let style = *self.style();
        *self = Empty(style);
    }

    pub fn is_empty(&self) -> bool {
        if let Empty(_) = *self { true } else { false }
    }

    pub fn is_char_extension(&self) -> bool {
        if let Extension(..) = *self { true } else { false }
    }

}

impl Default for CharCell {
    fn default() -> CharCell {
        Empty(Styles::default())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MimeType {
}
