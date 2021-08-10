/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

pub use tokenstream::*;

pub use crate::Token;

#[derive(PartialEq, Clone, Debug)]
pub enum Term {
    Var(String),
    Call(Box<Term>, Vec<Box<Term>>),
    Fn(Vec<String>, Box<Term>),
    Let(String, Box<Term>, Box<Term>),
}

#[derive(PartialEq, Clone, Debug)]
pub enum TypeVar {
    Unbound(i32, i32),
    Link(Box<Type>),
    Generic(i32),
}

#[derive(PartialEq, Clone, Debug)]
pub enum Type {
    Const(String),
    App(Box<Type>, Vec<Box<Type>>),
    Arrow(Vec<Type>, Box<Type>),
    Var(TypeVar),
}

fn ident() -> Parser<'static, Token, String> {
    Parser::new(|s| {
        s.next().and_then(|t| {
            if let Token::Ident(ident) = t {
                Some(ident)
            } else {
                None
            }
        })
    })
}

fn token(t: Token) -> Parser<'static, Token, Token> {
    satisfy(move |x| x == t)
}

fn simple_term() -> Parser<'static, Token, Term> {
    fmap(Term::Var, ident())
        | term().between(token(Token::Lparen), token(Token::Rbracket))
        | bind(simple_term(), |f| {
            bind(
                fmap(Box::new, term())
                    .sep_by(token(Token::Comma))
                    .between(token(Token::Lparen), token(Token::Rparen)),
                move |args| Parser::pure(Term::Call(Box::new(f.clone()), args)),
            )
        })
        | simple_term().between(token(Token::Lparen), token(Token::Rparen))
}

pub fn term() -> Parser<'static, Token, Term> {
    simple_term()
        | bind(
            token(Token::Let) >> ident() << token(Token::Equals),
            |ident| {
                bind(term() << token(Token::In), move |assign| {
                    let ident = ident.clone();
                    bind(term(), move |term| {
                        Parser::pure(Term::Let(
                            ident.clone(),
                            Box::new(assign.clone()),
                            Box::new(term),
                        ))
                    })
                })
            },
        )
        | bind(
            token(Token::Fn) >> ident().sep_by(token(Token::Spacing)) << token(Token::Arrow),
            |idents| {
                bind(term(), move |term| {
                    Parser::pure(Term::Fn(idents.clone(), Box::new(term)))
                })
            },
        )
}

fn simple_type() -> Parser<'static, Token, Type> {
    fmap(Type::Const, ident())
        | bind(simple_type(), |f| {
            bind(
                fmap(Box::new, r#type())
                    .sep_by(token(Token::Comma))
                    .between(token(Token::Lparen), token(Token::Rparen)),
                move |args| Parser::pure(Type::App(Box::new(f.clone()), args)),
            )
        })
        | r#type().between(token(Token::Lparen), token(Token::Rparen))
}

pub fn r#type() -> Parser<'static, Token, Type> {
    simple_type()
        | fmap(
            |ty| Type::Arrow(vec![], Box::new(ty)),
            (token(Token::Lparen) >> token(Token::Rparen) >> token(Token::Arrow)) >> r#type(),
        )
        | bind(simple_type() << token(Token::Arrow), |tl| {
            fmap(
                move |tr| Type::Arrow(vec![tl.clone()], Box::new(tr)),
                r#type(),
            )
        })
        | bind(
            (r#type() << token(Token::Comma))
                & (r#type().sep_by(token(Token::Comma)))
                    .between(token(Token::Lparen), token(Token::Rparen)),
            |tl| {
                fmap(
                    move |tr| Type::Arrow(tl.clone(), Box::new(tr)),
                    token(Token::Arrow) >> r#type(),
                )
            },
        )
}
