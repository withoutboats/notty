#[derive(Copy, Clone)]
pub struct Modifiers {
    pub shift: bool,
    pub caps: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl Modifiers {
    pub fn new() -> Modifiers {
        Modifiers {
            shift: false,
            caps: false,
            ctrl: false,
            alt: false
        }
    }

    pub fn triplet(&self) -> (bool, bool, bool) {
        (self.shift || self.caps, self.ctrl, self.alt)
    }

}
