/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod lexer;
pub use lexer::Lexer;

mod token;
pub use token::Token;

mod expr;
pub use expr::Expr;

mod ty;
pub use ty::Type;

mod stream;
pub use stream::Stream;

pub struct Parser<'a> {
    stream: Stream<'a, Token>,
}

impl<'a> Parser<'a> {
    pub fn new<I>(tokens: I) -> Self
    where
        I: Iterator<Item = Token> + 'a,
    {
        Self {
            stream: Stream::new(tokens),
        }
    }

    pub fn wtf(&mut self) -> Option<Expr> {
        if let Some(_) = self.stream.next() {
            Some(Expr::Wtf)
        } else {
            None
        }
    }
}

#[rustfmt::skip]
impl<'a> Iterator for Parser<'a> {
    type Item = Expr;
    fn next(&mut self) -> Option<Self::Item> {
        self.stream.next().and_then(|t| {
            self.stream.undo(1);
            let token = self.wtf().or_else(
                     || panic!("Unexpected token '{:?}'", t));
            self.stream.commit();
            token
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Parser, Lexer};

    #[test]
    fn wtf() {
        let s = "forall.a->a";
        let l = Lexer::new(s.chars());
        let p = Parser::new(l);
        for e in p {
            println!("{:?}", e);
        }
    }
}
