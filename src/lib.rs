#![feature(box_syntax, box_into_raw_non_null)]

use std::marker::PhantomData;
use std::mem;
use std::ptr::NonNull;

pub struct List<'a, T: 'a> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    marker: PhantomData<&'a Node<T>>,
}

pub struct Node<T> {
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
}

#[cfg(test)]
mod test {
    use super::List;
    #[test]
    fn basics() {
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
}
