/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::{Parser, Stream};

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

pub fn satisfy<'a, From: 'a>(f: impl Fn(From) -> bool + 'a) -> Parser<'a, From, From>
where
    From: Clone,
{
    Parser::new(move |s: &mut Stream<From>| {
        s.next().and_then(|x| {
            if f(x.clone()) {
                Some(x)
            } else {
                s.undo(1);
                None
            }
        })
    })
}

pub fn some<'a, From: 'a, To: 'a>(p: Parser<'a, From, To>) -> Parser<'a, From, Vec<To>>
where
    From: Clone,
    To: Clone,
{
    Parser::new(move |s| {
        let mut out = Vec::new();
        while let Some(x) = p.run(s) {
            out.push(x);
        }
        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    })
}

pub fn many<'a, From: 'a, To: 'a>(p: Parser<'a, From, To>) -> Parser<'a, From, Vec<To>>
where
    From: Clone,
    To: Clone,
{
    some(p) | Parser::pure(Vec::new())
}

// infix operators as members for utility
impl<'a, From: 'a, To: 'a> Parser<'a, From, To>
where
    From: Clone,
    To: Clone,
{
    pub fn sep_by1<Sep: 'a>(self, p: Parser<'a, From, Sep>) -> Parser<'a, From, Vec<To>>
    where
        Sep: Clone,
    {
        self.clone() & (many(p >> self))
    }

    pub fn sep_by<Sep: 'a>(self, p: Parser<'a, From, Sep>) -> Parser<'a, From, Vec<To>>
    where
        Sep: Clone,
    {
        self.sep_by1(p) | Parser::pure(Vec::new())
    }

    pub fn end_by<Sep: 'a>(self, p: Parser<'a, From, Sep>) -> Parser<'a, From, Vec<To>>
    where
        Sep: Clone,
    {
        many(self << p)
    }

    pub fn between<ToL: 'a, ToR: 'a>(
        self,
        l: Parser<'a, From, ToL>,
        r: Parser<'a, From, ToR>,
    ) -> Parser<'a, From, To>
    where
        ToL: Clone,
        ToR: Clone,
    {
        l >> self << r
    }
}
