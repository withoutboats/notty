//  notty is a new kind of terminal emulator.
//  Copyright (C) 2015 without boats
//  
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//  
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU Affero General Public License for more details.
//  
//  You should have received a copy of the GNU Affero General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
use std::str;

#[derive(Default)]
pub struct Attachments {
    data: Vec<u8>,
    points: Vec<usize>,
}

impl Attachments {

    pub fn clear(&mut self) {
        self.data.clear();
        self.points.clear();
    }

    pub fn iter(&self) -> AttachmentIter {
        self.into_iter()
    }

    pub fn append(&mut self, buf: &[u8], offset: &mut usize) -> Option<usize> {
        let mut offset_tmp = *offset + 1;
        'len: loop {
            match buf.get(offset_tmp).map(|&x|x) {
                Some(b'0'...b'9') | Some(b'A'...b'F') | Some(b'a'...b'f') => offset_tmp += 1,
                // According to the spec, the first byte after the length header must be ';'.
                // However, in order to continue making some sort of progress on malformed data,
                // any byte that is not a hexadigit will be treated as the terminal byte.
                Some(_) => break,
                None    => return None,
            }
        }
        let len = usize::from_str_radix(unsafe {
            str::from_utf8_unchecked(&buf[*offset + 1 .. offset_tmp])
        }, 16).unwrap();
        offset_tmp += 1;
        if offset_tmp + len > buf.len() {
            let data = &buf[offset_tmp..];
            self.data.extend(data);
            *offset = buf.len();
            Some(len - data.len())
        } else {
            *offset = offset_tmp + len;
            self.push(&buf[offset_tmp..*offset]);
            Some(0)
        }
    }

    pub fn append_incomplete(&mut self, buf: &[u8], offset: &mut usize, rem: usize) -> usize {
        if *offset + rem > buf.len() {
            let data = &buf[*offset..];
            self.data.extend(data);
            *offset = buf.len();
            rem - data.len()
        } else {
            self.push(&buf[*offset..*offset + rem]);
            *offset += rem;
            0
        }
    }

    fn push(&mut self, data: &[u8]) {
        self.data.extend(data);
        self.points.push(self.data.len());
    }

}

impl<'a> IntoIterator for &'a Attachments {
    type Item = &'a [u8];
    type IntoIter = AttachmentIter<'a>;
    fn into_iter(self) -> AttachmentIter<'a> {
        AttachmentIter {
            attachments: self,
            ctr: 0,
            prev: Some(0),
        }
    }
}

pub struct AttachmentIter<'a>{
    attachments: &'a Attachments,
    ctr: usize,
    prev: Option<usize>,
}

impl<'a> Iterator for AttachmentIter<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<&'a [u8]> {
        match self.prev.take() {
            Some(begin) => match self.attachments.points.get(self.ctr) {
                Some(&end)  => {
                    self.ctr += 1;
                    self.prev = Some(end);
                    Some(&self.attachments.data[begin..end])
                }
                None        => {
                    self.ctr += 1;
                    self.prev = None;
                    Some(&self.attachments.data[begin..])
                }
            },
            None        => None,
        }
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    static BLOCKS: &'static [&'static str] = &[
        "YELLOW SUBMARINE",
        "Hello, world!",
        "History is a nightmare from which I am trying to awaken.",
        "#",
        "Happy familie are all alike; every unhappy family is unhappy in its own way.",
        "--A little more test data.--"
    ];
    
    #[test]
    fn iterates() {
        let mut attachments = Attachments::default();
        for block in BLOCKS {
            attachments.push(block.as_bytes());
        }
        for (attachment, block) in (&attachments).into_iter().zip(BLOCKS) {
            assert_eq!(attachment, block.as_bytes());
        }
    }

    #[test]
    fn appends() {
        let buf = b"{10;YELLOW SUBMARINE{d;HELLO, ";
        let mut offset = 0;
        let mut attachments = Attachments::default();
        assert_eq!(attachments.append(buf, &mut offset), Some(0));
        assert_eq!(offset, 20);
        assert_eq!(attachments.append(buf, &mut offset), Some(6));
        assert_eq!(offset, buf.len());
        let buf = b"WORLD!}";
        let mut offset = 0;
        assert_eq!(attachments.append_incomplete(buf, &mut offset, 6), 0);
        assert_eq!(offset, buf.len() - 1);
        assert_eq!(buf[offset], b'}');
        for (attachment, block) in attachments.iter().zip(&["YELLOW SUBMARINE", "HELLO, WORLD!"]) {
            assert_eq!(attachment, block.as_bytes());
        }
    }

}
