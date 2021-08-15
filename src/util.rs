/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::Token;
use tokenstream::{fmap, many, satisfy, some, Parser};

macro_rules! extract {
    ($pattern:pat => $extracted_value:expr) => {
        Parser::new(|s| {
            s.next().and_then(|t| match t {
                $pattern => Some($extracted_value),
                _ => None,
            })
        })
    };
}

#[macro_export]
macro_rules! lazy {
    ($parser:expr) => {
        Parser::new(move |s| $parser.run(s))
    };
}

pub fn token(t: Token) -> Parser<'static, Token, Token> {
    satisfy(move |x| x == t)
}
pub fn ident() -> Parser<'static, Token, String> {
    extract!(Token::Ident(ident) => ident)
}
pub fn number() -> Parser<'static, Token, String> {
    extract!(Token::Number(number) => number)
}
pub fn comment() -> Parser<'static, Token, String> {
    fmap(
        |c| c[..2].to_owned(),
        extract!(Token::Comment(comment) => comment),
    )
}
pub fn many_space() -> Parser<'static, Token, ()> {
    fmap(|_| {}, many(token(Token::Spacing) | token(Token::Newline)))
}
pub fn some_space() -> Parser<'static, Token, ()> {
    fmap(|_| {}, some(token(Token::Spacing) | token(Token::Newline)))
}
pub fn eof() -> Parser<'static, Token, ()> {
    Parser::new(|s| {
        if let Some(_) = s.next() {
            None
        } else {
            Some(())
        }
    })
}

pub fn spaced<'a, To: 'a>(p: Parser<'a, Token, To>) -> Parser<'a, Token, To>
where
    To: Clone,
{
    many_space() >> p << many_space()
}
pub fn parens<'a, To: 'a>(p: Parser<'a, Token, To>) -> Parser<'a, Token, To>
where
    To: Clone,
{
    p.between(token(Token::Lparen), token(Token::Rparen))
}
pub fn braces<'a, To: 'a>(p: Parser<'a, Token, To>) -> Parser<'a, Token, To>
where
    To: Clone,
{
    p.between(token(Token::Lbrace), token(Token::Rbrace))
}
pub fn brackets<'a, To: 'a>(p: Parser<'a, Token, To>) -> Parser<'a, Token, To>
where
    To: Clone,
{
    p.between(token(Token::Lbracket), token(Token::Rbracket))
}
pub fn comma_list<'a, To: 'a>(p: Parser<'a, Token, To>) -> Parser<'a, Token, Vec<To>>
where
    To: Clone,
{
    p.sep_by(token(Token::Comma))
}
pub fn comma_list1<'a, To: 'a>(p: Parser<'a, Token, To>) -> Parser<'a, Token, Vec<To>>
where
    To: Clone,
{
    p.sep_by1(token(Token::Comma))
}
