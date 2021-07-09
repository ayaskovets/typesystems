/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::Stream;

impl<'a, T> Stream<'a, T>
where
    T: Clone,
{
    pub fn fallback<Any>(&mut self, len: usize) -> Option<Any> {
        self.undo(len);
        None
    }

    pub fn squash<P>(&mut self, predicate: P)
    where
        P: Fn(&T) -> bool,
    {
        loop {
            if let Some(t) = self.next() {
                if predicate(&t) {
                    continue;
                } else {
                    self.undo(1);
                    break;
                }
            }
            break;
        }
    }

    pub fn take_while<P>(&mut self, predicate: P) -> Vec<T>
    where
        P: Fn(&T) -> bool,
    {
        let mut taken: Vec<T> = Vec::new();
        loop {
            if let Some(t) = self.next() {
                if predicate(&t) {
                    taken.push(t);
                    continue;
                } else {
                    self.undo(1);
                    break;
                }
            }
            break;
        }
        taken
    }
}
