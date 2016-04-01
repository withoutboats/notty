use std::collections::VecDeque;
use std::mem;

const E_NOT_EMPTY: &'static str = "The Ring deque can't be empty if it is not None.";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ring<T> {
    pub top: T,
    rest: Option<VecDeque<T>>,
}

impl<T> Ring<T> {

    pub fn new(top: T) -> Ring<T> {
        Ring {
            top: top,
            rest: None
        }
    }

    pub fn len(&self) -> usize {
        if let Some(ref rest) = self.rest {
            rest.len() + 1
        } else { 1 }
    }
    
    pub fn iter(&self) -> Iter<T> {
        Iter {
            top: Some(&self.top),
            rest: self.rest.as_ref().map(|rest| rest.into_iter()),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            top: Some(&mut self.top),
            rest: self.rest.as_mut().map(|rest| rest.into_iter()),
        }
    }

    pub fn push(&mut self, top: T) {
        let previous_top = mem::replace(&mut self.top, top);
        if self.rest.is_some() { self.rest.as_mut().unwrap().push_back(previous_top); }
        else {
            let mut ring = VecDeque::new();
            ring.push_back(previous_top);
            self.rest = Some(ring);
        }
    }

    pub fn pop(&mut self) {
        if let Some(mut ring) = self.rest.take() {
            self.top = ring.pop_back().expect(E_NOT_EMPTY);
            if !ring.is_empty() {
                self.rest = Some(ring);
            }
        }
    }

    pub fn rotate_down(&mut self) {
        if let Ring { ref mut top, rest: Some(ref mut rest) } = *self {
            let previous_top = mem::replace(top, rest.pop_back().expect(E_NOT_EMPTY));
            rest.push_front(previous_top);
        }
    }

    pub fn rotate_up(&mut self) {
        if let Ring { ref mut top, rest: Some(ref mut rest) } = *self {
            let previous_top = mem::replace(top, rest.pop_front().expect(E_NOT_EMPTY));
            rest.push_back(previous_top);
        }
    }

}

impl<'a, T> IntoIterator for &'a Ring<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Ring<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;
    fn into_iter(self) -> IterMut<'a, T> {
        self.iter_mut()
    }
}

impl<T> IntoIterator for Ring<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            top: Some(self.top),
            rest: self.rest.map(|rest| rest.into_iter()),
        }
    }
}

impl<T> Extend<T> for Ring<T> {
    fn extend<I>(&mut self, iterable: I) where I: IntoIterator<Item=T> {
        for elem in iterable {
            self.push(elem);
        }
    }
}

pub struct Iter<'a, T: 'a> {
    top: Option<&'a T>,
    rest: Option<<&'a VecDeque<T> as IntoIterator>::IntoIter>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        self.top.take().or_else(|| self.rest.as_mut().and_then(|iter| iter.next_back()))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> {
        self.rest.as_mut().and_then(|iter| iter.next()).or_else(|| self.top.take())
    }
}

pub struct IterMut<'a, T: 'a> {
    top: Option<&'a mut T>,
    rest: Option<<&'a mut VecDeque<T> as IntoIterator>::IntoIter>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<&'a mut T> {
        self.top.take().or_else(|| self.rest.as_mut().and_then(|iter| iter.next_back()))
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<&'a mut T> {
        self.rest.as_mut().and_then(|iter| iter.next()).or_else(|| self.top.take())
    }
}

pub struct IntoIter<T> {
    top: Option<T>,
    rest: Option<<VecDeque<T> as IntoIterator>::IntoIter>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.top.take().or_else(|| self.rest.as_mut().and_then(|iter| iter.next_back()))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.rest.as_mut().and_then(|iter| iter.next()).or_else(|| self.top.take())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::fmt::Debug;

    macro_rules! deque {
        [$($arg:expr),*] => {
            {
                let mut deque = ::std::collections::VecDeque::new();
                $( deque.push_back($arg); )*
                deque
            }
        }
    }

    fn one_ring() -> Ring<i32> {
        Ring {
            top: 0,
            rest: None,
        }
    }

    fn three_ring() -> Ring<i32> {
        Ring {
            top: 2,
            rest: Some(deque![0, 1]),
        }
    }

    fn run_test<F, T>(f: F, res: [T; 2]) where F: Fn(Ring<i32>) -> T, T: PartialEq + Debug {
        assert_eq!(f(one_ring()), res[0]);
        assert_eq!(f(three_ring()), res[1]);
    }

    #[test]
    fn new() {
        assert_eq!(Ring::new(0), one_ring())
    }

    #[test]
    fn push() {
        run_test(|mut ring| { ring.push(5); ring }, [
            Ring { top: 5, rest: Some(deque![0]) },
            Ring { top: 5, rest: Some(deque![0, 1, 2]) },
        ])
    }

    #[test]
    fn pop() {
        run_test(|mut ring| { ring.pop(); ring }, [
            Ring { top: 0, rest: None },
            Ring { top: 1, rest: Some(deque![0]) },
        ]);
    }

    #[test]
    fn rotate_down() {
        run_test(|mut ring| { ring.rotate_down(); ring }, [
            Ring { top: 0, rest: None },
            Ring { top: 1, rest: Some(deque![2, 0]) },
        ]);
    }

    #[test]
    fn rotate_up() {
        run_test(|mut ring| { ring.rotate_up(); ring }, [
            Ring { top: 0, rest: None },
            Ring { top: 0, rest: Some(deque![1, 2]) },
        ]);
    }

    #[test]
    fn iter() {
        run_test(|ring| ring.iter().cloned().collect::<Vec<i32>>(), [
            vec![0],
            vec![2, 1, 0],
        ])
    }

    #[test]
    fn iter_rev() {
        run_test(|ring| ring.iter().rev().cloned().collect::<Vec<i32>>(), [
            vec![0],
            vec![0, 1, 2],
        ])
    }

    #[test]
    fn iter_mut() {
        run_test(|mut ring| ring.iter_mut().map(|&mut x| x).collect::<Vec<i32>>(), [
            vec![0],
            vec![2, 1, 0],
        ])
    }

    #[test]
    fn iter_mut_rev() {
        run_test(|mut ring| ring.iter_mut().rev().map(|&mut x| x).collect::<Vec<i32>>(), [
            vec![0],
            vec![0, 1, 2],
        ])
    }

    #[test]
    fn into_iter() {
        run_test(|ring| ring.into_iter().collect::<Vec<i32>>(), [
            vec![0],
            vec![2, 1, 0],
        ])
    }

    #[test]
    fn into_iter_rev() {
        run_test(|ring| ring.into_iter().rev().collect::<Vec<i32>>(), [
            vec![0],
            vec![0, 1, 2],
        ])
    }

}
