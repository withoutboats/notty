#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct Coords {
    pub x: u32,
    pub y: u32,
}

impl Coords {
    pub fn new(x: u32, y: u32) -> Coords {
        Coords {
            x: x,
            y: y,
        }
    }

}
