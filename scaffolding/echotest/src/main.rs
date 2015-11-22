use std::io::Write;

extern crate hocus_pocus;

use hocus_pocus::LineBuffer;

fn main() {
    let mut buffer = LineBuffer::new(String::from("Echo >>")).unwrap();
    let mut string = String::new();
    loop {
        match buffer.read_line(&mut string).unwrap() {
            0   => break,
            _   => buffer.write_all(string.as_bytes()).unwrap(),
        }
        string.clear();
    }
    buffer.write_all("\n".as_bytes()).unwrap();
}
