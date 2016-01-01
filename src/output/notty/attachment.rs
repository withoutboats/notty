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
use base64;

pub struct Attachments {
    data: Vec<String>,
}

impl Default for Attachments {
    fn default() -> Attachments {
        Attachments { data: vec![String::new()] }
    }
}

impl Attachments {

    pub fn clear(&mut self) {
        self.data.truncate(1);
        self.data.last_mut().unwrap().clear();
    }

    pub fn iter(&self) -> AttachmentIter {
        self.into_iter()
    }

    pub fn append(&mut self, ch: char) -> Option<bool> {
        match ch {
            '0'...'9' | 'A'...'Z' | 'a'...'z' | '+' | '/' | '=' => {
                self.data.last_mut().unwrap().push(ch);
                None
            }
            '#'                                                 => {
                self.data.push(String::new());
                None
            }
            '\u{9c}'                                            => Some(true),
            _                                                   => Some(false),
        }
    }

}

impl<'a> IntoIterator for &'a Attachments {
    type Item = Vec<u8>;
    type IntoIter = AttachmentIter<'a>;
    fn into_iter(self) -> AttachmentIter<'a> {
        AttachmentIter {
            attachments: self.data.iter(),
        }
    }
}

pub struct AttachmentIter<'a>{
    attachments: <&'a Vec<String> as IntoIterator>::IntoIter,
}

impl<'a> Iterator for AttachmentIter<'a> {
    type Item = Vec<u8>;
    fn next(&mut self) -> Option<Vec<u8>> {
        self.attachments.next().and_then(|data| base64::u8de(data.as_bytes()).ok())
    }
}

#[cfg(test)]
mod tests {

    use base64;
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
        let attachments = Attachments {
            data: BLOCKS.iter().map(|s| base64::encode(s).unwrap()).collect()
        };
        for (attachment, block) in (&attachments).into_iter().zip(BLOCKS) {
            assert_eq!(attachment, block.as_bytes());
        }
    }

    #[test]
    fn appends() {
        let mut attachments = Attachments::default();
        for data in BLOCKS.iter().map(|s| base64::encode(s).unwrap()) {
            for ch in data.chars() {
                assert_eq!(attachments.append(ch), None);
            }
            assert_eq!(attachments.append('#'), None);
        }
        assert_eq!(attachments.append('\u{9c}'), Some(true));
        for (attachment, block) in (&attachments).into_iter().zip(BLOCKS) {
            assert_eq!(attachment, block.as_bytes());
        }
    }

    #[test]
    fn wont_append_invalid_chars() {
        let mut attachments = Attachments::default();
        assert_eq!(attachments.append('~'), Some(false));
        assert_eq!(attachments.data.last().unwrap().len(), 0);
    }

}
