/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::{Combinator, Stream, Tokenizer};

pub type Lexer<'a, Token> = Tokenizer<'a, char, Token>;
pub type Parser<'a, Token, Expr> = Tokenizer<'a, Token, Expr>;

impl<'a, T> Stream<'a, T>
where
    T: Clone,
{
    pub fn skip<P>(&mut self, predicate: P)
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

    pub fn take<P>(&mut self, predicate: P) -> Vec<T>
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

impl<'a, From> Stream<'a, From>
where
    From: Clone,
{
    pub fn run<To>(&mut self, combinator: Combinator<From, To>) -> Option<To> {
        let len = self.len();
        (combinator.f)(self).or_else(|| {
            self.undo(self.len() - len);
            None
        })
    }
}
