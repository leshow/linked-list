#![feature(box_into_raw_non_null)]

use std::marker::PhantomData;
use std::ptr::NonNull;

pub struct List<'a, T: 'a> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    marker: PhantomData<&'a Node<T>>,
}

struct Node<T> {
    elem: T,
    next: Option<NonNull<Node<T>>>,
}

impl<'a, T> List<'a, T> {
    pub fn new() -> Self {
        List {
            head: None,
            tail: None,
            marker: PhantomData,
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
        self.tail = Some(node);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|head| unsafe {
            self.head = (*head.as_ptr()).next;
            if self.head.is_none() {
                self.head = None;
            }
            Box::from_raw(head.as_ptr()).elem
        })
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

    pub fn into_iter(self) -> IntoIter<'a, T> {
        IntoIter(self)
    }

    pub fn iter(&self) -> Iter<'a, T> {
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
pub struct IntoIter<'a, T: 'a>(List<'a, T>);

pub struct Iter<'a, T: 'a> {
    next: Option<&'a Node<T>>,
}

pub struct IterMut<'a, T: 'a> {
    next: Option<&'a mut Node<T>>,
}

impl<'a, T> Iterator for IntoIter<'a, T> {
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

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn pop_push() {
        let mut list = List::new();
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
        let mut list = List::new();
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
        let mut list = List::new();
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
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);
        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(&mut 1));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), None);
    }
}
