/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::rc::Rc;

use crate::Stream;

pub type Output<To> = Option<Vec<To>>;
#[macro_export]
macro_rules! out {
    () => {
        None
    };
    ($($outputs:expr),*) => {
        Some(vec![$($outputs),*])
    };
}

#[derive(Clone)]
pub struct Combinator<'a, From, To>
where
    From: Clone,
{
    pub f: Rc<dyn Fn(&mut Stream<From>) -> Output<To> + 'a>,
}

impl<'a, From, To> Combinator<'a, From, To>
where
    From: Clone,
{
    pub fn new<F: 'a>(f: F) -> Self
    where
        F: Fn(&mut Stream<From>) -> Output<To>,
    {
        Self { f: Rc::new(f) }
    }
}

impl<'a, From: 'a, To: 'a> std::ops::Shl for &Combinator<'a, From, To>
where
    From: Clone,
{
    type Output = Combinator<'a, From, To>;
    fn shl(self, rhs: &Combinator<'a, From, To>) -> Self::Output {
        let lf = self.f.clone();
        let rf = rhs.f.clone();
        Combinator {
            f: Rc::new(move |s| lf(s).and_then(|x| rf(s).and_then(|_| Some(x)))),
        }
    }
}

impl<'a, From: 'a, To: 'a> std::ops::Shr for &Combinator<'a, From, To>
where
    From: Clone,
{
    type Output = Combinator<'a, From, To>;
    fn shr(self, rhs: &Combinator<'a, From, To>) -> Self::Output {
        let lf = self.f.clone();
        let rf = rhs.f.clone();
        Combinator {
            f: Rc::new(move |s| lf(s).and_then(|_| rf(s))),
        }
    }
}

impl<'a, From: 'a, To: 'a> std::ops::BitOr for &Combinator<'a, From, To>
where
    From: Clone,
{
    type Output = Combinator<'a, From, To>;
    fn bitor(self, rhs: &Combinator<'a, From, To>) -> Self::Output {
        let lf = self.f.clone();
        let rf = rhs.f.clone();
        Combinator {
            f: Rc::new(move |s| lf(s).or_else(|| rf(s))),
        }
    }
}

impl<'a, From: 'a, To: 'a> std::ops::BitAnd for &Combinator<'a, From, To>
where
    From: Clone,
{
    type Output = Combinator<'a, From, To>;
    fn bitand(self, rhs: &Combinator<'a, From, To>) -> Self::Output {
        let lf = self.f.clone();
        let rf = rhs.f.clone();
        Combinator {
            f: Rc::new(move |s| {
                lf(s).and_then(|mut x| {
                    rf(s).and_then(|mut y| {
                        x.append(&mut y);
                        Some(x)
                    })
                })
            }),
        }
    }
}
