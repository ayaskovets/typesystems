/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::VecDeque;

pub struct Stream<'a, T> {
    iter: Box<dyn Iterator<Item = T> + 'a>,
    undo: VecDeque<T>,
    len: usize,
}

impl<'a, T> Stream<'a, T> {
    pub fn new<I: Iterator<Item = T> + 'a>(iter: I) -> Self {
        Self {
            iter: Box::new(iter),
            undo: VecDeque::new(),
            len: 0,
        }
    }

    pub fn undo(&mut self, times: usize) {
        self.len = std::cmp::min(self.len + times, self.undo.len());
    }

    pub fn commit(&mut self) {
        self.undo.truncate(self.len);
        self.len = self.undo.len();
    }
}

impl<'a, T: Copy> Iterator for Stream<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.len -= 1;
            return Some(self.undo[self.len]);
        }

        self.iter.next().and_then(|t| {
            self.undo.push_front(t);
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
        s.undo(2);
        assert_eq!(s.next(), Some('b'));
        assert_eq!(s.next(), Some('c'));
    }

    #[test]
    fn commit() {
        let mut s = Stream::new("abcd".chars());
        assert_eq!(s.next(), Some('a'));
        assert_eq!(s.next(), Some('b'));
        assert_eq!(s.next(), Some('c'));
        s.undo(1);
        s.commit();
        assert_eq!(s.next(), Some('c'));
        assert_eq!(s.next(), Some('d'));
    }

    #[test]
    fn overflow() {
        let mut s = Stream::new("ab".chars());
        assert_eq!(s.next(), Some('a'));
        assert_eq!(s.next(), Some('b'));
        assert_eq!(s.next(), None);
        assert_eq!(s.next(), None);
        s.undo(1);
        assert_eq!(s.next(), Some('b'));
        assert_eq!(s.next(), None);
        s.commit();
        s.undo(42);
        assert_eq!(s.next(), None);
    }
}
