/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use tokenstream::{bind, fmap, Parser, Stream, Tokenizer};

use crate::{brackets, comma_list1, eof, ident, lazy, many_space, parens, spaced, token, Token};

#[derive(PartialEq, Clone, Debug)]
pub enum Ty {
    Const(String),
    App(Box<Ty>, Vec<Ty>),
    Arrow(Vec<Ty>, Box<Ty>),
    Forall(Vec<String>, Box<Ty>),
}

impl std::str::FromStr for Ty {
    type Err = ();
    fn from_str(s: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        let mut lexer = Stream::new(Tokenizer::new(s.chars()));
        let parser = (ty() << eof()) | (forall() << eof());
        parser.run(&mut lexer).ok_or(())
    }
}

impl std::fmt::Display for Ty {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Ty::Const(name) => {
                write!(fmt, "{}", name)
            }
            Ty::App(ty, params) => {
                write!(fmt, "{}[", ty)?;
                if !params.is_empty() {
                    write!(fmt, "{}", params[0])?;
                }
                for i in 1..params.len() {
                    write!(fmt, ", {}", params[i])?;
                }
                write!(fmt, "]")
            }
            Ty::Arrow(init, tail) => {
                match init.len() {
                    0 => {
                        write!(fmt, "() -> ")?;
                    }
                    1 => match init[0] {
                        Ty::Const(_) | Ty::App(_, _) => {
                            write!(fmt, "{} -> ", init[0])?;
                        }
                        _ => {
                            write!(fmt, "({}) -> ", init[0])?;
                        }
                    },
                    len @ _ => {
                        write!(fmt, "({}", init[0])?;
                        for i in 1..len {
                            write!(fmt, ", {}", init[i])?;
                        }
                        write!(fmt, ") -> ")?;
                    }
                };
                write!(fmt, "{}", tail)
            }
            Ty::Forall(params, arrow) => {
                write!(fmt, "forall[")?;
                if !params.is_empty() {
                    write!(fmt, "{}", params[0])?;
                }
                for i in 1..params.len() {
                    write!(fmt, ", {}", params[i])?;
                }
                write!(fmt, "] {}", arrow)
            }
        }
    }
}

fn simple_ty() -> Parser<'static, Token, Ty> {
    let p_const = fmap(Ty::Const, ident());
    let p_ty_parens = lazy!(parens(spaced(ty())));

    fn ty_params() -> Parser<'static, Token, Vec<Ty>> {
        brackets(spaced(comma_list1(spaced(ty()))))
    }

    bind(
        bind(p_const, move |name| {
            let _name = name.clone();
            fmap(
                move |params| Ty::App(Box::new(_name.clone()), params),
                ty_params(),
            ) | Parser::pure(name)
        }) | bind(p_ty_parens, move |ty| {
            let _ty = ty.clone();
            fmap(
                move |params| Ty::App(Box::new(_ty.clone()), params),
                ty_params(),
            ) | Parser::pure(ty)
        }),
        move |app| {
            let _app = app.clone();
            fmap(
                move |params| Ty::App(Box::new(_app.clone()), params),
                ty_params(),
            ) | Parser::pure(app)
        },
    )
}

fn ty() -> Parser<'static, Token, Ty> {
    let p_arrow = bind(
        fmap(|_| vec![], token(Token::Lparen) >> token(Token::Rparen))
            | fmap(|ty| vec![ty], simple_ty())
            | lazy!(parens(
                (spaced(ty()) << token(Token::Comma)) & spaced(comma_list1(spaced(ty())))
            )),
        move |init| {
            fmap(
                move |tail| Ty::Arrow(init.clone(), Box::new(tail)),
                spaced(token(Token::Arrow)) >> ty(),
            )
        },
    );

    p_arrow | simple_ty()
}

fn forall() -> Parser<'static, Token, Ty> {
    let p_forall = bind(
        (token(Token::Forall) >> token(Token::Lbracket))
            >> (spaced(comma_list1(spaced(ident()))) << token(Token::Rbracket)),
        move |params| {
            fmap(
                move |ty| Ty::Forall(params.clone(), Box::new(ty)),
                many_space() >> ty(),
            )
        },
    );

    p_forall | ty()
}

#[cfg(test)]
mod tests {
    use super::{Ty, Ty::*};

    use std::str::FromStr;

    fn collect(s: &str) -> Option<Ty> {
        Ty::from_str(s).ok()
    }

    #[test]
    fn invalid() {
        assert_eq!(collect(""), None);
        assert_eq!(collect("a b"), None);
        assert_eq!(collect("t[]"), None);
        assert_eq!(collect("[]"), None);
        assert_eq!(collect("->"), None);
        assert_eq!(collect("() -> t[]"), None);
    }

    #[test]
    fn type_const() {
        assert_eq!(collect("int"), Some(Const(String::from("int"))));
        assert_eq!(collect("((int ) )"), Some(Const(String::from("int"))));
    }

    #[test]
    fn arrow() {
        assert_eq!(
            collect("() -> b"),
            Some(Arrow(vec![], Box::new(Const(String::from("b")))))
        );
        assert_eq!(
            collect("() -> a -> b"),
            Some(Arrow(
                vec![],
                Box::new(Arrow(
                    vec![Const(String::from("a"))],
                    Box::new(Const(String::from("b")))
                ))
            ))
        );
        assert_eq!(
            collect("(( () -> a ) -> b)"),
            Some(Arrow(
                vec![Arrow(vec![], Box::new(Const(String::from("a"))))],
                Box::new(Const(String::from("b")))
            ))
        );
        assert_eq!(
            collect("a -> ( a , b ) -> b"),
            Some(Arrow(
                vec![Const(String::from("a")),],
                Box::new(Arrow(
                    vec![Const(String::from("a")), Const(String::from("b"))],
                    Box::new(Const(String::from("b")))
                ))
            ))
        );
    }

    #[test]
    fn application() {
        assert_eq!(
            collect("t[a]"),
            Some(App(
                Box::new(Const(String::from("t"))),
                vec![Const(String::from("a"))]
            ))
        );
        assert_eq!(
            collect("t[ a , ( b ) ]"),
            Some(App(
                Box::new(Const(String::from("t"))),
                vec![Const(String::from("a")), Const(String::from("b"))]
            ))
        );
        assert_eq!(
            collect("t[() -> a, b[(c, d) -> c]]"),
            Some(App(
                Box::new(Const(String::from("t"))),
                vec![
                    Arrow(vec![], Box::new(Const(String::from("a")))),
                    App(
                        Box::new(Const(String::from("b"))),
                        vec![Arrow(
                            vec![Const(String::from("c")), Const(String::from("d"))],
                            Box::new(Const(String::from("c")))
                        )]
                    )
                ]
            ))
        );
    }

    #[test]
    fn forall() {
        assert_eq!(
            collect("forall[a, b] (a -> b, c) -> a"),
            Some(Forall(
                vec![String::from("a"), String::from("b")],
                Box::new(Arrow(
                    vec![
                        Arrow(
                            vec![Const(String::from("a"))],
                            Box::new(Const(String::from("b")))
                        ),
                        Const(String::from("c"))
                    ],
                    Box::new(Const(String::from("a")))
                ))
            ))
        );
    }
}
