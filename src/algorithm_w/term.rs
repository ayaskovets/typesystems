/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use tokenstream::{bind, fmap, Parser};

use crate::token::Token;
use crate::util::{comma_list1, ident, many_space, parens, spaced, token};

#[derive(PartialEq, Clone, Debug)]
pub enum Term {
    Var(String),
    Call(Box<Term>, Vec<Box<Term>>),
    Fn(Vec<String>, Box<Term>),
    Let(String, Box<Term>, Box<Term>),
}

fn simple_term() -> Parser<'static, Token, Term> {
    let p_var = fmap(Term::Var, ident());
    let p_term_parens = parens(spaced(term()));
    let p_call = bind(fmap(Box::new, simple_term()), |f| {
        fmap(
            move |args| Term::Call(f.clone(), args),
            parens(spaced(comma_list1(spaced(fmap(Box::new, term()))))),
        )
    });

    p_var | p_term_parens | p_call
}

pub fn term() -> Parser<'static, Token, Term> {
    let p_simple_term = simple_term();
    let p_let_in = bind(
        token(Token::Let) >> spaced(ident()) << token(Token::Equals),
        |ident| {
            bind(spaced(term()) << token(Token::In), move |assign| {
                let ident = ident.clone();
                fmap(
                    move |term| Term::Let(ident.clone(), Box::new(assign.clone()), Box::new(term)),
                    many_space() >> term(),
                )
            })
        },
    );
    let p_fn_arrow = bind(
        token(Token::Fn) >> spaced(ident().sep_by1(token(Token::Spacing))) << token(Token::Arrow),
        |idents| {
            fmap(
                move |term| Term::Fn(idents.clone(), Box::new(term)),
                spaced(term()),
            )
        },
    );

    p_simple_term | p_let_in | p_fn_arrow
}
