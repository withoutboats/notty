use std::cell::RefCell;

use datatypes::MediaPosition;
use mime::{TopLevel, SubLevel, Mime};
use terminal::interfaces::Resizeable;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Image {
    inner: RefCell<InnerImage>,
    pos: MediaPosition,
    width: u32,
    height: u32,
}

impl Image {
    pub fn new(data: Vec<u8>, mime: Mime, pos: MediaPosition, width: u32, height: u32) -> Image {
        Image {
            inner: RefCell::new(InnerImage::Encoded(EncodedData {
                data: data,
                format: ImageFormat::from_mime(mime).expect("TODO move this conversion to parser"),
            })),
            pos: pos,
            width: width,
            height: height,
        }
    }

    pub fn pos(&self) -> MediaPosition {
        self.pos
    }

    pub fn intern<F>(&self, intern: F) -> usize where F: FnOnce(&EncodedData) -> usize {
        let mut inner = self.inner.borrow_mut();
        let tag = match *inner {
            InnerImage::Encoded(ref data)   => intern(data),
            InnerImage::Interned(tag)       => return tag,
        };
        *inner = InnerImage::Interned(tag);
        tag
    }
}

impl Resizeable for Image {
    fn dims(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn resize_width(&mut self, width: u32) {
        self.width = width;
    }

    fn resize_height(&mut self, height: u32) {
        self.height = height;
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
enum InnerImage {
    Encoded(EncodedData),
    Interned(usize),
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct EncodedData {
    pub data: Vec<u8>,
    pub format: ImageFormat,
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum ImageFormat {
    Gif,
    Jpeg,
    Png,
}

impl ImageFormat {
    fn from_mime(mime: Mime) -> Option<ImageFormat> {
        match (mime.0, mime.1) {
            (TopLevel::Image, SubLevel::Gif)    => Some(ImageFormat::Gif),
            (TopLevel::Image, SubLevel::Jpeg)   => Some(ImageFormat::Jpeg),
            (TopLevel::Image, SubLevel::Png)    => Some(ImageFormat::Png),
            _                                   => None,
        }
    }
}
