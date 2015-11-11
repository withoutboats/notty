#[derive(Default)]
pub struct Attachments {
    data: Vec<u8>,
    points: Vec<usize>,
}

impl Attachments {

    pub fn push(&mut self, data: &[u8]) {
        self.points.push(self.data.len() + data.len());
        self.data.extend(data);
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.points.clear();
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
        for &block in BLOCKS {
            attachments.push(block.as_bytes());
        }
        for (attachment, block) in (&attachments).into_iter().zip(BLOCKS) {
            assert_eq!(attachment, block.as_bytes());
        }
    }
}
