/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::VecDeque;
use std::str::Chars;

pub struct Stream<'a> {
    cs: Chars<'a>,
    undo: VecDeque<char>,
    len: usize,
}

impl<'a> Stream<'a> {
    pub fn new(chars: Chars<'a>) -> Self {
        Self {
            cs: chars,
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

impl<'a> Iterator for Stream<'a> {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.len -= 1;
            return Some(self.undo[self.len]);
        }

        self.cs.next().and_then(|c| {
            self.undo.push_front(c);
            Some(c)
        })
    }
}
