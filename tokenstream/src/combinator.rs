/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::Stream;

pub struct Combinator<From, To>
where
    From: Clone,
{
    pub f: Box<dyn Fn(&mut Stream<From>) -> Option<To>>,
}

impl<From, To> Combinator<From, To>
where
    From: Clone,
{
    pub fn new<F: 'static>(f: F) -> Self
    where
        F: Fn(&mut Stream<From>) -> Option<To>,
    {
        Self { f: Box::new(f) }
    }
}

impl<From: 'static, To: 'static> std::ops::Shl for Combinator<From, To>
where
    From: Clone,
{
    type Output = Self;
    fn shl(self, rhs: Combinator<From, To>) -> Self::Output {
        Combinator {
            f: Box::new(move |s| (self.f)(s).and_then(|x| (rhs.f)(s).and_then(|_| Some(x)))),
        }
    }
}

impl<From: 'static, To: 'static> std::ops::Shr for Combinator<From, To>
where
    From: Clone,
{
    type Output = Self;
    fn shr(self, rhs: Combinator<From, To>) -> Self::Output {
        Combinator {
            f: Box::new(move |s| (self.f)(s).and_then(|_| (rhs.f)(s))),
        }
    }
}

impl<From: 'static, To: 'static> std::ops::BitOr for Combinator<From, To>
where
    From: Clone,
{
    type Output = Self;
    fn bitor(self, rhs: Combinator<From, To>) -> Self::Output {
        Combinator {
            f: Box::new(move |s| (self.f)(s).or_else(|| (rhs.f)(s))),
        }
    }
}

impl<From: 'static, To: 'static> std::ops::BitAnd for Combinator<From, To>
where
    From: Clone,
{
    type Output = Combinator<From, Vec<To>>;
    fn bitand(self, rhs: Combinator<From, To>) -> Self::Output {
        Combinator {
            f: Box::new(move |s| {
                (self.f)(s).and_then(|x| (rhs.f)(s).and_then(|y| Some(vec![x, y])))
            }),
        }
    }
}
