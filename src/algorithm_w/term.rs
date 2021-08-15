/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use tokenstream::{bind, fmap, Parser};

use crate::{comma_list, eof, ident, lazy, many_space, parens, spaced, token, Token};

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

    bind(
        bind(p_var, move |f| {
            let _f = f.clone();
            fmap(move |args| Term::Call(Box::new(f.clone()), args), args()) | Parser::pure(_f)
        }) | bind(p_term_parens, move |f| {
            let _f = f.clone();
            fmap(move |args| Term::Call(Box::new(f.clone()), args), args()) | Parser::pure(_f)
        }),
        |call| {
            let _call = call.clone();
            fmap(move |args| Term::Call(Box::new(call.clone()), args), args()) | Parser::pure(_call)
        },
    )
}

fn term() -> Parser<'static, Token, Term> {
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

    lazy!(simple_term()) | p_let_in | p_fn_arrow
}

pub fn term_eof() -> Parser<'static, Token, Term> {
    term() << eof()
}

#[cfg(test)]
mod tests {
    use super::{term_eof, Term, Term::*};
    use tokenstream::{Stream, Tokenizer};

    fn collect(s: &str) -> Option<Term> {
        let lexer = Tokenizer::new(s.chars());
        let mut lexer_stream = Stream::new(lexer);
        let parser = term_eof();

        parser.run(&mut lexer_stream)
    }

    #[test]
    fn invalid() {
        assert_eq!(collect(""), None);
        assert_eq!(collect("()"), None);
        assert_eq!(collect("a b"), None);
        assert_eq!(collect("a, b"), None);
        assert_eq!(collect("a = b"), None);
        assert_eq!(collect("let a = b in ()"), None);
        assert_eq!(collect("f(a"), None);
        assert_eq!(collect("(f a)"), None);
    }

    #[test]
    fn var() {
        assert_eq!(collect("a"), Some(Var(String::from("a"))));
        assert_eq!(collect("(((a)))"), Some(Var(String::from("a"))));
    }

    #[test]
    fn call() {
        assert_eq!(
            collect("(fn)((a), b)"),
            Some(Call(
                Box::new(Var(String::from("fn"))),
                vec![
                    Box::new(Var(String::from("a"))),
                    Box::new(Var(String::from("b")))
                ]
            ))
        );
        assert_eq!(
            collect("(f)(a, g(b))"),
            Some(Call(
                Box::new(Var(String::from("f"))),
                vec![
                    Box::new(Var(String::from("a"))),
                    Box::new(Call(
                        Box::new(Var(String::from("g"))),
                        vec![Box::new(Var(String::from("b")))]
                    ))
                ]
            ))
        );
        assert_eq!(
            collect("((((fn(a)))((b))))"),
            Some(Call(
                Box::new(Call(
                    Box::new(Var(String::from("fn"))),
                    vec![Box::new(Var(String::from("a")))]
                )),
                vec![Box::new(Var(String::from("b")))]
            ))
        );
        assert_eq!(
            collect("f(a)(g(b))"),
            Some(Call(
                Box::new(Call(
                    Box::new(Var(String::from("f"))),
                    vec![Box::new(Var(String::from("a")))]
                )),
                vec![Box::new(Call(
                    Box::new(Var(String::from("g"))),
                    vec![Box::new(Var(String::from("b")))]
                ))]
            ))
        );
    }

    #[test]
    fn f() {
        assert_eq!(
            collect(r"\x y -> x"),
            Some(Fn(
                vec![String::from("x"), String::from("y")],
                Box::new(Var(String::from("x")))
            ))
        );
    }

    #[test]
    fn let_in() {
        assert_eq!(
            collect(r"let c = \f g x -> f(g(x)) in c(f, g)"),
            Some(Let(
                String::from("c"),
                Box::new(Fn(
                    vec![String::from("f"), String::from("g"), String::from("x")],
                    Box::new(Call(
                        Box::new(Var(String::from("f"))),
                        vec![Box::new(Call(
                            Box::new(Var(String::from("g"))),
                            vec![Box::new(Var(String::from("x")))]
                        ))]
                    )),
                )),
                Box::new(Call(
                    Box::new(Var(String::from("c"))),
                    vec![
                        Box::new(Var(String::from("f"))),
                        Box::new(Var(String::from("g")))
                    ]
                ))
            ))
        );
        assert_eq!(
            collect("let a = x in let b = y in f(x, y)"),
            Some(Let(
                String::from("a"),
                Box::new(Var(String::from("x"))),
                Box::new(Let(
                    String::from("b"),
                    Box::new(Var(String::from("y"))),
                    Box::new(Call(
                        Box::new(Var(String::from("f"))),
                        vec![
                            Box::new(Var(String::from("x"))),
                            Box::new(Var(String::from("y")))
                        ]
                    ))
                ))
            ))
        )
    }
}
