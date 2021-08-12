/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use tokenstream::{bind, fmap, Parser};

use crate::{comma_list, ident, lazy, many_space, parens, spaced, token, Token};

#[derive(PartialEq, Clone, Debug)]
pub enum Term {
    Var(String),
    Call(Box<Term>, Vec<Box<Term>>),
    Fn(Vec<String>, Box<Term>),
    Let(String, Box<Term>, Box<Term>),
}

fn simple_term() -> Parser<'static, Token, Term> {
    let p_var = fmap(Term::Var, ident());
    let p_term_parens = lazy!(parens(spaced(term())));

    fn args() -> Parser<'static, Token, Vec<Box<Term>>> {
        parens(spaced(comma_list(spaced(fmap(Box::new, term())))))
    }

    bind(p_var, move |f| {
        let _f = f.clone();
        fmap(move |args| Term::Call(Box::new(f.clone()), args), args()) | Parser::pure(_f)
    }) | bind(p_term_parens, move |f| {
        let _f = f.clone();
        fmap(move |args| Term::Call(Box::new(f.clone()), args), args()) | Parser::pure(_f)
    })
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
        token(Token::Fn) >> spaced(ident().sep_by1(many_space())) << token(Token::Arrow),
        |idents| {
            fmap(
                move |term| Term::Fn(idents.clone(), Box::new(term)),
                many_space() >> term(),
            )
        },
    );

    p_simple_term | p_let_in | p_fn_arrow
}
