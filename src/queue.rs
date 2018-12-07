use std::{iter::FromIterator, marker::PhantomData, mem, ptr::NonNull};

#[derive(Debug, Hash)]
pub struct Queue<T> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    _marker: PhantomData<Box<Node<T>>>,
    len: usize,
}

#[derive(Debug, Hash)]
struct Node<T> {
    elem: T,
    next: Option<NonNull<Node<T>>>,
}

impl<T: Default> Default for Queue<T> {
    fn default() -> Self {
        Queue::new()
    }
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Queue {
            head: None,
            tail: None,
            _marker: PhantomData,
            len: 0,
        }
    }

    // push to tail
    pub fn push(&mut self, elem: T) {
        let node = Box::into_raw_non_null(Box::new(Node {
            elem,
            next: None, // None b/c pushing to tail
        }));
        match self.tail {
            Some(mut old_tail) => unsafe {
                old_tail.as_mut().next = Some(node);
            },
            None => {
                self.head = Some(node);
            }
        };
        self.len += 1;
        self.tail = Some(node);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|head| unsafe {
            self.head = (*head.as_ptr()).next;
            if self.head.is_none() {
                self.head = None;
            }
            self.len -= 1;
            Box::from_raw(head.as_ptr()).elem
        })
    }

    pub fn append(&mut self, right: &mut Queue<T>) {
        match self.tail {
            None => mem::swap(self, right),
            Some(tail) => {
                unsafe {
                    (*tail.as_ptr()).next = right.head;
                }
                self.tail = right.tail.take();
                self.len += mem::replace(&mut right.len, 0);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn peek(&self) -> Option<&T> {
        self.head
            .as_ref()
            .map(|head| unsafe { &(*head.as_ptr()).elem })
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head
            .as_mut()
            .map(|head| unsafe { &mut (*head.as_ptr()).elem })
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            next: self.head.as_ref().map(|h| unsafe { &(*h.as_ptr()) }),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            next: self.head.as_mut().map(|h| unsafe { &mut (*h.as_ptr()) }),
        }
    }
}

impl<T> IntoIterator for Queue<T> {
    type IntoIter = IntoIter<T>;
    type Item = T;

    fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

#[derive(Debug)]
pub struct IntoIter<T>(Queue<T>);

#[derive(Debug)]
pub struct Iter<'a, T: 'a> {
    next: Option<&'a Node<T>>,
}

#[derive(Debug)]
pub struct IterMut<'a, T: 'a> {
    next: Option<&'a mut Node<T>>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_ref().map(|n| unsafe { &(*n.as_ptr()) });
            &node.elem
        })
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| {
            self.next = node.next.as_mut().map(|n| unsafe { &mut (*n.as_ptr()) });
            &mut node.elem
        })
    }
}

impl<'a, A> Extend<A> for Queue<A> {
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T) {
        for elem in iter {
            self.push(elem);
        }
    }
}

impl<'a, T> FromIterator<T> for Queue<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut q = Queue::new();
        q.extend(iter);
        q
    }
}

#[cfg(test)]
mod test {
    use super::Queue;

    #[test]
    fn pop_push() {
        let mut list = Queue::new();
        assert_eq!(list.pop(), None);

        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));

        list.push(4);
        list.push(5);

        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn into_iter() {
        let mut list = Queue::new();
        list.push(1);
        list.push(2);
        list.push(3);
        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter() {
        let mut list = Queue::new();
        list.push(1);
        list.push(2);
        list.push(3);
        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_mut() {
        let mut list = Queue::new();
        list.push(1);
        list.push(2);
        list.push(3);
        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(&mut 1));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn append() {
        let mut a = Queue::new();
        a.push(1);
        a.push(2);
        a.push(3);

        let mut b = Queue::new();
        b.push(4);
        b.push(5);
        a.append(&mut b);
        let mut iter = a.into_iter();

        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), Some(5));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn extend() {
        let mut a = Queue::new();
        a.push(1);
        a.push(2);
        a.push(3);

        let mut b = Queue::new();
        b.push(4);
        b.push(5);
        a.extend(b);
        let mut iter = a.into_iter();

        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), Some(5));
        assert_eq!(iter.next(), None);
    }
}
