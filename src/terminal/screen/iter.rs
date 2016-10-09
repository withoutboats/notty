use super::section::ScreenSection;
use super::panel::Panel::*;

/// An iterator over all of the visible panels in the terminal's screen.
pub struct Panels<'a, T: 'a> {
    pub(super) stack: Vec<&'a ScreenSection<T>>,
}

impl<'a, T> Iterator for Panels<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        fn dfs<'a, T>(section: &'a ScreenSection<T>,
                        stack: &mut Vec<&'a ScreenSection<T>>) -> &'a T {
            match *section.top() {
                Fill(ref fill)  => fill,
                Split(ref split) => {
                    let (left, right) = split.children();
                    stack.push(right);
                    dfs(left, stack)
                }
                _ => unreachable!()
            }
        }
        self.stack.pop().map(|section| dfs(section, &mut self.stack))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }

}

impl<'a, T> ExactSizeIterator for Panels<'a, T> {
    fn len(&self) -> usize {
        self.stack.iter().cloned().map(ScreenSection::count_leaves).sum()
    }
}
