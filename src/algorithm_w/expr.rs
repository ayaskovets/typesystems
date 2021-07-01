/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

pub use super::Token;
pub use parser_stream::{Stream, Streamable};

#[derive(PartialEq, Clone, Debug)]
enum Expr {
    Var(String),
    Call(Box<Expr>, Vec<Box<Expr>>),
    Fn(Vec<String>, Box<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
}

#[derive(PartialEq, Clone, Debug)]
enum TypeVar {
    Unbound(i32, i32),
    Link(Box<Type>),
    Generic(i32),
}

#[derive(PartialEq, Clone, Debug)]
enum Type {
    Const(String),
    App(Box<Type>, Vec<Box<Type>>),
    Arrow(Vec<Type>, Box<Type>),
    Var(TypeVar),
}

#[derive(PartialEq, Clone, Debug)]
pub enum Total {
    Expr(Expr),
    Type(Type),
}

impl Streamable<Token> for Total {
    #[rustfmt::skip]
    fn from(s: &mut Stream<Token>) -> Option<Total> {
           expr(s).or_else(
        || r#type(s))
    }
}

fn expr(s: &mut Stream<Token>) -> Option<Total> {

}

fn r#type(s: &mut Stream<Token>) -> Option<Total> {

}
