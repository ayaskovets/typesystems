/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::Stream;

#[derive(Clone)]
pub struct Parser<'a, From, To>
where
    From: Clone,
    To: Clone,
{
    f: std::rc::Rc<dyn Fn(&mut Stream<From>) -> Option<To> + 'a>,
}

impl<'a, From, To: 'a> Parser<'a, From, To>
where
    From: Clone,
    To: Clone,
{
    pub fn new(f: impl Fn(&mut Stream<From>) -> Option<To> + 'a) -> Self {
        Parser {
            f: std::rc::Rc::new(f),
        }
    }

    // Alternative: empty
    pub fn empty() -> Self {
        Parser::new(move |_| None)
    }

    pub fn run(&self, s: &mut Stream<From>) -> Option<To> {
        let state = s.len();
        (self.f)(s).or_else(|| {
            s.undo(s.len() - state);
            None
        })
    }
}

impl<'a, From, To: 'a> Parser<'a, From, To>
where
    From: Clone,
    To: Clone,
{
    // Applicative: pure
    pub fn pure(any: To) -> Self {
        Parser::new(move |_| Some(any.clone()))
    }
}

// Functor: fmap(f, p)
pub fn fmap<'a, From: 'a, ToA: 'a, ToB: 'a>(
    f: impl Fn(ToA) -> ToB + 'a,
    p: Parser<'a, From, ToA>,
) -> Parser<'a, From, ToB>
where
    From: Clone,
    ToA: Clone,
    ToB: Clone,
{
    Parser::new(move |s| p.run(s).and_then(|a| Some(f(a))))
}

// Applicative: l << r
impl<'a, From: 'a, ToL: 'a, ToR: 'a> std::ops::Shl<Parser<'a, From, ToR>> for Parser<'a, From, ToL>
where
    From: Clone,
    ToL: Clone,
    ToR: Clone,
{
    type Output = Parser<'a, From, ToL>;
    fn shl(self, rhs: Parser<'a, From, ToR>) -> Self::Output {
        Parser::new(move |s| self.run(s).and_then(|l| rhs.run(s).and_then(|_| Some(l))))
    }
}

// Applicative: l >> r
impl<'a, From: 'a, ToL: 'a, ToR: 'a> std::ops::Shr<Parser<'a, From, ToR>> for Parser<'a, From, ToL>
where
    From: Clone,
    ToL: Clone,
    ToR: Clone,
{
    type Output = Parser<'a, From, ToR>;
    fn shr(self, rhs: Parser<'a, From, ToR>) -> Self::Output {
        Parser::new(move |s| self.run(s).and_then(|_| rhs.run(s).and_then(|r| Some(r))))
    }
}

// Alternative: l | r
impl<'a, From: 'a, To: 'a> std::ops::BitOr<Parser<'a, From, To>> for Parser<'a, From, To>
where
    From: Clone,
    To: Clone,
{
    type Output = Parser<'a, From, To>;
    fn bitor(self, rhs: Parser<'a, From, To>) -> Self::Output {
        Parser::new(move |s| self.run(s).or_else(|| rhs.run(s)))
    }
}

// Monad: bind(p, f)
pub fn bind<'a, From: 'a, ToA: 'a, ToB: 'a>(
    p: Parser<'a, From, ToA>,
    f: impl Fn(ToA) -> Parser<'a, From, ToB> + 'a,
) -> Parser<'a, From, ToB>
where
    From: Clone,
    ToA: Clone,
    ToB: Clone,
{
    Parser::new(move |s| p.run(s).and_then(|a| f(a).run(s)))
}

// Monadic bind for merging results: l & r
impl<'a, From: 'a, To: 'a> std::ops::BitAnd<Parser<'a, From, Vec<To>>> for Parser<'a, From, Vec<To>>
where
    From: Clone,
    To: Clone,
{
    type Output = Parser<'a, From, Vec<To>>;
    fn bitand(self, rhs: Parser<'a, From, Vec<To>>) -> Self::Output {
        bind(self, move |xs| {
            bind(rhs.clone(), move |ys| {
                let mut out = xs.clone();
                out.extend(ys);
                Parser::pure(out)
            })
        })
    }
}

impl<'a, From: 'a, To: 'a> std::ops::BitAnd<Parser<'a, From, Vec<To>>> for Parser<'a, From, To>
where
    From: Clone,
    To: Clone,
{
    type Output = Parser<'a, From, Vec<To>>;
    fn bitand(self, rhs: Parser<'a, From, Vec<To>>) -> Self::Output {
        bind(self, move |x| {
            bind(rhs.clone(), move |xs| {
                let mut out = vec![x.clone()];
                out.extend(xs);
                Parser::pure(out)
            })
        })
    }
}

impl<'a, From: 'a, To: 'a> std::ops::BitAnd<Parser<'a, From, To>> for Parser<'a, From, Vec<To>>
where
    From: Clone,
    To: Clone,
{
    type Output = Parser<'a, From, Vec<To>>;
    fn bitand(self, rhs: Parser<'a, From, To>) -> Self::Output {
        bind(self, move |xs| {
            bind(rhs.clone(), move |x| {
                let mut out = xs.clone();
                out.push(x);
                Parser::pure(out)
            })
        })
    }
}
