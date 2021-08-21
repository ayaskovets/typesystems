/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use tokenstream::{bind, fmap, Parser, Stream, Tokenizer};

use crate::{comma_list, eof, ident, lazy, many_space, parens, spaced, token, Token};

#[derive(PartialEq, Clone, Debug)]
pub enum Term {
    Var(String),
    Let(String, Box<Term>, Box<Term>),
    Abs(Vec<String>, Box<Term>),
    App(Box<Term>, Vec<Term>),
}

impl std::str::FromStr for Term {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        let mut lexer = Stream::new(Tokenizer::new(s.chars()));
        let parser = term() << eof();
        parser
            .run(&mut lexer)
            .ok_or(format!("Error while parsing {}", s))
    }
}

impl std::fmt::Display for Term {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Term::Var(name) => {
                write!(fmt, "{}", name)
            }
            Term::Let(name, assign, body) => {
                write!(fmt, "let {} = {} in {}", name, assign, body)
            }
            Term::Abs(args, body) => {
                write!(fmt, "\\")?;
                if !args.is_empty() {
                    write!(fmt, "{}", args[0])?;
                }
                for i in 1..args.len() {
                    write!(fmt, " {}", args[i])?;
                }
                write!(fmt, " -> {}", body)
            }
            Term::App(f, args) => {
                write!(fmt, "{}(", f)?;
                if !args.is_empty() {
                    write!(fmt, "{}", args[0])?;
                }
                for i in 1..args.len() {
                    write!(fmt, ", {}", args[i])?;
                }
                write!(fmt, ")")
            }
        }
    }
}

fn name() -> Parser<'static, Token, String> {
    ident()
        | fmap(|_| String::from("true"), token(Token::True))
        | fmap(|_| String::from("false"), token(Token::False))
        | fmap(|_| String::from("not"), token(Token::Not))
        | fmap(|_| String::from("and"), token(Token::And))
        | fmap(|_| String::from("or"), token(Token::Or))
}

fn simple_term() -> Parser<'static, Token, Term> {
    let p_var = fmap(Term::Var, name());
    let p_term_parens = lazy!(parens(spaced(term())));

    fn args() -> Parser<'static, Token, Vec<Term>> {
        parens(spaced(comma_list(spaced(term()))))
    }

    bind(
        bind(p_var, |f| {
            let _f = f.clone();
            fmap(move |args| Term::App(Box::new(f.clone()), args), args()) | Parser::pure(_f)
        }) | bind(p_term_parens, |f| {
            let _f = f.clone();
            fmap(move |args| Term::App(Box::new(f.clone()), args), args()) | Parser::pure(_f)
        }),
        |app| {
            let _call = app.clone();
            fmap(move |args| Term::App(Box::new(app.clone()), args), args()) | Parser::pure(_call)
        },
    )
}

fn term() -> Parser<'static, Token, Term> {
    let p_let = bind(
        token(Token::Let) >> spaced(name()) << token(Token::Equals),
        |name| {
            bind(spaced(term()) << token(Token::In), move |assign| {
                let name = name.clone();
                fmap(
                    move |body| Term::Let(name.clone(), Box::new(assign.clone()), Box::new(body)),
                    many_space() >> term(),
                )
            })
        },
    );
    let p_fn = bind(
        token(Token::Backslash) >> spaced(name().sep_by1(many_space())) << token(Token::Arrow),
        |args| {
            fmap(
                move |body| Term::Abs(args.clone(), Box::new(body)),
                many_space() >> term(),
            )
        },
    );

    p_let | p_fn | simple_term()
}

#[cfg(test)]
mod tests {
    use super::{Term, Term::*};

    use std::str::FromStr;

    fn collect(s: &str) -> Option<Term> {
        Term::from_str(s).ok()
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
    fn let_in() {
        assert_eq!(
            collect(r"let c = \f g x -> f(g(x)) in c(f, g)"),
            Some(Let(
                String::from("c"),
                Box::new(Abs(
                    vec![String::from("f"), String::from("g"), String::from("x")],
                    Box::new(App(
                        Box::new(Var(String::from("f"))),
                        vec![App(
                            Box::new(Var(String::from("g"))),
                            vec![Var(String::from("x"))]
                        )]
                    )),
                )),
                Box::new(App(
                    Box::new(Var(String::from("c"))),
                    vec![Var(String::from("f")), Var(String::from("g"))]
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
                    Box::new(App(
                        Box::new(Var(String::from("f"))),
                        vec![Var(String::from("x")), Var(String::from("y"))]
                    ))
                ))
            ))
        )
    }

    #[test]
    fn abs() {
        assert_eq!(
            collect(r"\x y -> x"),
            Some(Abs(
                vec![String::from("x"), String::from("y")],
                Box::new(Var(String::from("x")))
            ))
        );
    }

    #[test]
    fn app() {
        assert_eq!(
            collect("(fn)((a), b)"),
            Some(App(
                Box::new(Var(String::from("fn"))),
                vec![Var(String::from("a")), Var(String::from("b"))]
            ))
        );
        assert_eq!(
            collect("(f)(a, g(b))"),
            Some(App(
                Box::new(Var(String::from("f"))),
                vec![
                    Var(String::from("a")),
                    App(
                        Box::new(Var(String::from("g"))),
                        vec![Var(String::from("b"))]
                    )
                ]
            ))
        );
        assert_eq!(
            collect("((((fn(a)))((b))))"),
            Some(App(
                Box::new(App(
                    Box::new(Var(String::from("fn"))),
                    vec![Var(String::from("a"))]
                )),
                vec![Var(String::from("b"))]
            ))
        );
        assert_eq!(
            collect("f(a)(g(b))"),
            Some(App(
                Box::new(App(
                    Box::new(Var(String::from("f"))),
                    vec![Var(String::from("a"))]
                )),
                vec![App(
                    Box::new(Var(String::from("g"))),
                    vec![Var(String::from("b"))]
                )]
            ))
        );
    }
}
