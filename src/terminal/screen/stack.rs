use std::mem;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stack<T> {
    pub top: T,
    rest: Option<Vec<T>>,
}

impl<T> Stack<T> {

    pub fn new(top: T) -> Stack<T> {
        Stack {
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
        if self.rest.is_some() { self.rest.as_mut().unwrap().push(previous_top); }
        else { self.rest = Some(vec![previous_top]); }
    }

    pub fn pop(&mut self) {
        if let Some(mut stack) = self.rest.take() {
            self.top = stack.pop().expect("If the stack vec exists it can't be empty.");
            if !stack.is_empty() {
                self.rest = Some(stack);
            }
        }
    }

}

impl<'a, T> IntoIterator for &'a Stack<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Stack<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;
    fn into_iter(self) -> IterMut<'a, T> {
        self.iter_mut()
    }
}

impl<T> IntoIterator for Stack<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            top: Some(self.top),
            rest: self.rest.map(|rest| rest.into_iter()),
        }
    }
}

impl<T> Extend<T> for Stack<T> {
    fn extend<I>(&mut self, iterable: I) where I: IntoIterator<Item=T> {
        for elem in iterable {
            self.push(elem);
        }
    }
}

pub struct Iter<'a, T: 'a> {
    top: Option<&'a T>,
    rest: Option<<&'a Vec<T> as IntoIterator>::IntoIter>,
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
    rest: Option<<&'a mut Vec<T> as IntoIterator>::IntoIter>,
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
    rest: Option<<Vec<T> as IntoIterator>::IntoIter>,
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

    fn one_stack() -> Stack<i32> {
        Stack {
            top: 0,
            rest: None,
        }
    }

    fn three_stack() -> Stack<i32> {
        Stack {
            top: 2,
            rest: Some(vec![0, 1]),
        }
    }

    fn run_test<F, T>(f: F, res: [T; 2]) where F: Fn(Stack<i32>) -> T, T: PartialEq + Debug {
        assert_eq!(f(one_stack()), res[0]);
        assert_eq!(f(three_stack()), res[1]);
    }

    #[test]
    fn new() {
        assert_eq!(Stack::new(0), one_stack())
    }

    #[test]
    fn push() {
        run_test(|mut stack| { stack.push(5); stack }, [
            Stack { top: 5, rest: Some(vec![0]) },
            Stack { top: 5, rest: Some(vec![0, 1, 2]) },
        ])
    }

    #[test]
    fn pop() {
        run_test(|mut stack| { stack.pop(); stack }, [
            Stack { top: 0, rest: None },
            Stack { top: 1, rest: Some(vec![0]) },
        ]);
    }

    #[test]
    fn iter() {
        run_test(|stack| stack.iter().cloned().collect::<Vec<i32>>(), [
            vec![0],
            vec![2, 1, 0],
        ])
    }

    #[test]
    fn iter_rev() {
        run_test(|stack| stack.iter().rev().cloned().collect::<Vec<i32>>(), [
            vec![0],
            vec![0, 1, 2],
        ])
    }

    #[test]
    fn iter_mut() {
        run_test(|mut stack| stack.iter_mut().map(|&mut x| x).collect::<Vec<i32>>(), [
            vec![0],
            vec![2, 1, 0],
        ])
    }

    #[test]
    fn iter_mut_rev() {
        run_test(|mut stack| stack.iter_mut().rev().map(|&mut x| x).collect::<Vec<i32>>(), [
            vec![0],
            vec![0, 1, 2],
        ])
    }

    #[test]
    fn into_iter() {
        run_test(|stack| stack.into_iter().collect::<Vec<i32>>(), [
            vec![0],
            vec![2, 1, 0],
        ])
    }

    #[test]
    fn into_iter_rev() {
        run_test(|stack| stack.into_iter().rev().collect::<Vec<i32>>(), [
            vec![0],
            vec![0, 1, 2],
        ])
    }

}
