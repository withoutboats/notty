use std::mem;

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
        self.top.take().or_else(|| self.rest.as_mut().and_then(|iter| iter.next()))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> {
        self.rest.as_mut().and_then(|iter| iter.next_back()).or_else(|| self.top.take())
    }
}

pub struct IterMut<'a, T: 'a> {
    top: Option<&'a mut T>,
    rest: Option<<&'a mut Vec<T> as IntoIterator>::IntoIter>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<&'a mut T> {
        self.top.take().or_else(|| self.rest.as_mut().and_then(|iter| iter.next()))
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<&'a mut T> {
        self.rest.as_mut().and_then(|iter| iter.next_back()).or_else(|| self.top.take())
    }
}

pub struct IntoIter<T> {
    top: Option<T>,
    rest: Option<<Vec<T> as IntoIterator>::IntoIter>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.top.take().or_else(|| self.rest.as_mut().and_then(|iter| iter.next()))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.rest.as_mut().and_then(|iter| iter.next_back()).or_else(|| self.top.take())
    }
}
