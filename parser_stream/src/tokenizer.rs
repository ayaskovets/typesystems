/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::stream::{Stream, Streamable};

pub struct Tokenizer<'a, From, To>
where
    From: Clone + std::fmt::Debug,
    To: Streamable<From>,
{
    from_stream: Stream<'a, From>,
    to_t: std::marker::PhantomData<To>,
}

impl<'a, From, To> Tokenizer<'a, From, To>
where
    From: Clone + std::fmt::Debug,
    To: Streamable<From>,
{
    pub fn new<I>(tokens: I) -> Self
    where
        I: Iterator<Item = From> + 'a,
    {
        Self {
            from_stream: Stream::new(tokens),
            to_t: std::marker::PhantomData::<To>,
        }
    }
}

impl<'a, From, To> Iterator for Tokenizer<'a, From, To>
where
    From: Clone + std::fmt::Debug,
    To: Streamable<From>,
{
    type Item = To;
    fn next(&mut self) -> Option<Self::Item> {
        self.from_stream.next().and_then(|t| {
            self.from_stream.undo(1);
            match To::from(&mut self.from_stream) {
                None => panic!("Unexpected token '{:?}'", t),
                token => {
                    self.from_stream.commit();
                    token
                }
            }
        })
    }
}
