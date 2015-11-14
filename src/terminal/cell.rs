use image::DynamicImage;

use datatypes::{Coords, MediaPosition};
use terminal::Styles;

use self::CharCell::*;

#[derive(Clone)]
pub enum CharCell {
    Empty(Styles),
    Char(char, Styles),
    Grapheme(String, Styles),
    Image {
        data: DynamicImage,
        pos: MediaPosition,
        end: Coords,
        style: Styles,
    },
    Extension(Coords, Styles),
}

impl CharCell {

    pub fn character(ch: char, style: Styles) -> CharCell {
        Char(ch, style)
    }

    pub fn grapheme(grapheme: String, style: Styles) -> CharCell {
       Grapheme(grapheme, style)
    }

    pub fn image(data: DynamicImage, pos: MediaPosition, end: Coords, style: Styles) -> CharCell {
        Image {
            data: data,
            pos: pos,
            end: end,
            style: style,
        }
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

    pub fn repr(&self) -> String {
        match *self {
            Char(c, _)          => c.to_string(),
            Grapheme(ref s, _)  => s.clone(),
            Image { .. }        => String::from("IMG"),
            Empty(_)            => String::new(),
            Extension(..)       => String::from("EXT"),
        }
    }

    pub fn style(&self) -> &Styles {
        match *self {
            Char(_, ref style)
                | Grapheme(_, ref style)
                | Empty(ref style)
                | Image { ref style, .. }
                | Extension(_, ref style)
                => style
        }
    }

    pub fn style_mut(&mut self) -> &mut Styles {
        match *self {
            Char(_, ref mut style)
                | Grapheme(_, ref mut style)
                | Empty(ref mut style)
                | Image { ref mut style, .. }
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
