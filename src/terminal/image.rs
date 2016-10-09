use datatypes::MediaPosition;
use mime::Mime;
use terminal::interfaces::Resizeable;

pub struct ImageData {
    pub data: Vec<u8>,
    pub mime: Mime,
    pub pos: MediaPosition,
    pub width: u32,
    pub height: u32,
}

impl Resizeable for ImageData {
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
