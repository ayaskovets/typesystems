/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::VecDeque;
use std::vec::Vec;

pub struct Stream<'a, T>
where
    T: Clone,
{
    iter: Box<dyn Iterator<Item = T> + 'a>,
    undo: VecDeque<T>,
    cursor: usize,
}

impl<'a, T> Stream<'a, T>
where
    T: Clone,
{
    pub fn new<I: Iterator<Item = T> + 'a>(iter: I) -> Self {
        Self {
            iter: Box::new(iter),
            undo: VecDeque::new(),
            cursor: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.undo.len() - self.cursor
    }

    pub fn undo(&mut self, times: usize) {
        self.cursor = std::cmp::min(self.cursor + times, self.undo.len());
    }

    pub fn commit(&mut self) {
        self.undo.truncate(self.cursor);
        self.cursor = self.undo.len();
    }

    pub fn wrap(&mut self) -> Vec<T> {
        let read = self.undo.drain(self.cursor..).rev().collect();
        self.commit();
        read
    }
}

impl<'a, T> Iterator for Stream<'a, T>
where
    T: Clone,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor > 0 {
            self.cursor -= 1;
            return Some(self.undo[self.cursor].clone());
        }

        self.iter.next().and_then(|t| {
            self.undo.push_front(t.clone());
            Some(t)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Stream;

    #[test]
    fn undo() {
        let mut s = Stream::new("abc".chars());
        assert_eq!(s.next(), Some('a'));
        assert_eq!(s.next(), Some('b'));
        assert_eq!(s.next(), Some('c'));
        assert_eq!(s.len(), 3);
        s.undo(2);
        assert_eq!(s.len(), 1);
        assert_eq!(s.next(), Some('b'));
        assert_eq!(s.next(), Some('c'));
        assert_eq!(s.len(), 3);
        assert_eq!(s.wrap(), vec!['a', 'b', 'c']);
    }

    #[test]
    fn commit() {
        let mut s = Stream::new("abcd".chars());
        assert_eq!(s.len(), 0);
        assert_eq!(s.next(), Some('a'));
        assert_eq!(s.next(), Some('b'));
        assert_eq!(s.next(), Some('c'));
        assert_eq!(s.len(), 3);
        s.undo(1);
        assert_eq!(s.wrap(), vec!['a', 'b']);
        assert_eq!(s.len(), 0);
        assert_eq!(s.next(), Some('c'));
        assert_eq!(s.next(), Some('d'));
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn overflow() {
        let mut s = Stream::new("ab".chars());
        assert_eq!(s.len(), 0);
        assert_eq!(s.next(), Some('a'));
        assert_eq!(s.next(), Some('b'));
        assert_eq!(s.len(), 2);
        assert_eq!(s.next(), None);
        assert_eq!(s.len(), 2);
        assert_eq!(s.next(), None);
        assert_eq!(s.len(), 2);
        s.undo(1);
        assert_eq!(s.next(), Some('b'));
        assert_eq!(s.len(), 2);
        assert_eq!(s.next(), None);
        assert_eq!(s.len(), 2);
        assert_eq!(s.wrap(), vec!['a', 'b']);
        assert_eq!(s.len(), 0);
        s.undo(42);
        assert_eq!(s.len(), 0);
        assert_eq!(s.next(), None);
        assert_eq!(s.len(), 0);
    }
}
