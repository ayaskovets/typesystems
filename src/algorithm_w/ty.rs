/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;
use std::str::FromStr;

use tokenstream::{bind, fmap, Parser, Stream, Tokenizer};

use crate::{Bindable, Env, Gen, Id, Level};

use crate::{brackets, comma_list1, eof, ident, lazy, many_space, parens, spaced, token, Token};

#[derive(PartialEq, Clone, Debug)]
pub enum Ty {
    Const(String),
    App(Box<Ty>, Vec<Ty>),
    Arrow(Vec<Ty>, Box<Ty>),
    Forall(Vec<String>, Box<Ty>),
}

impl std::str::FromStr for Ty {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        let mut lexer = Stream::new(Tokenizer::new(s.chars()));
        let parser = (ty() << eof()) | (forall() << eof());
        parser
            .run(&mut lexer)
            .ok_or(format!("Error while parsing {}", s))
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
        assert_eq!(format!("{}", collect("int").unwrap()), String::from("int"));
        assert_eq!(collect("((int ) )"), Some(Const(String::from("int"))));
        assert_eq!(
            format!("{}", collect("((int ) )").unwrap()),
            String::from("int")
        );
    }

    #[test]
    fn arrow() {
        assert_eq!(
            collect("() -> b"),
            Some(Arrow(vec![], Box::new(Const(String::from("b")))))
        );
        assert_eq!(
            format!("{}", collect("() -> b").unwrap()),
            String::from("() -> b")
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
            format!("{}", collect("() -> a -> b").unwrap()),
            String::from("() -> a -> b")
        );
        assert_eq!(
            collect("(( () -> a ) -> b)"),
            Some(Arrow(
                vec![Arrow(vec![], Box::new(Const(String::from("a"))))],
                Box::new(Const(String::from("b")))
            ))
        );
        assert_eq!(
            format!("{}", collect("(( () -> a ) -> b)").unwrap()),
            String::from("(() -> a) -> b")
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
        assert_eq!(
            format!("{}", collect("a -> ( a , b ) -> b").unwrap()),
            String::from("a -> (a, b) -> b")
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
            format!("{}", collect("t[a]").unwrap()),
            String::from("t[a]")
        );
        assert_eq!(
            collect("t[ a , ( b ) ]"),
            Some(App(
                Box::new(Const(String::from("t"))),
                vec![Const(String::from("a")), Const(String::from("b"))]
            ))
        );
        assert_eq!(
            format!("{}", collect("t[ a , ( b ) ]").unwrap()),
            String::from("t[a, b]")
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
        assert_eq!(
            format!("{}", collect("t[() -> a, b[(c, d) -> c]]").unwrap()),
            String::from("t[() -> a, b[(c, d) -> c]]")
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
        assert_eq!(
            format!("{}", collect("forall[a, b] (a -> b, c) -> a").unwrap()),
            String::from("forall[a, b] (a -> b, c) -> a")
        );
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum Type {
    Const(String),
    App(Box<Type>, Vec<Type>),
    Arrow(Vec<Type>, Box<Type>),
    TypeVar(Id, Level),
    Generic(Id),
}

impl From<(Id, Option<Level>)> for Type {
    fn from((id, level): (Id, Option<Level>)) -> Self {
        if let Some(level) = level {
            Type::TypeVar(id, level)
        } else {
            Type::Generic(id)
        }
    }
}

impl Bindable for Type {
    fn get_unbound_id_level(t: &Self) -> Option<(Id, Level)> {
        if let Type::TypeVar(id, level) = t {
            Some((*id, *level))
        } else {
            None
        }
    }
}

impl Type {
    fn from(ty: Ty, gen: &mut Gen<Type>) -> Self {
        let mut env = Env::new();
        match ty {
            Ty::Const(name) => Type::Const(name),
            Ty::App(ty, params) => Type::App(
                Box::new(Type::from(*ty, gen)),
                params
                    .into_iter()
                    .map(|t_param| Type::from(t_param, gen))
                    .collect(),
            ),
            Ty::Arrow(init, tail) => Type::Arrow(
                init.into_iter()
                    .map(|t_param| Type::from(t_param, gen))
                    .collect(),
                Box::new(Type::from(*tail, gen)),
            ),
            Ty::Forall(params, arrow) => {
                for param in params {
                    let t_param = gen.newvar(None);
                    env.insert(&param, t_param);
                }

                fn lookup(ty: Ty, env: &Env<Type>) -> Type {
                    match ty {
                        Ty::Const(name) => env
                            .lookup(&name)
                            .map(|t_const| t_const.clone())
                            .unwrap_or(Type::Const(name)),
                        Ty::App(ty, params) => Type::App(
                            Box::new(lookup(*ty, env)),
                            params
                                .into_iter()
                                .map(|t_param| lookup(t_param, env))
                                .collect(),
                        ),
                        Ty::Arrow(init, tail) => Type::Arrow(
                            init.into_iter()
                                .map(|t_param| lookup(t_param, env))
                                .collect(),
                            Box::new(lookup(*tail, env)),
                        ),
                        Ty::Forall(_, _) => unreachable!("Nested forall is not possible"),
                    }
                }

                lookup(*arrow, &mut env)
            }
        }
    }

    pub fn from_str(s: &str, gen: &mut Gen<Type>) -> std::result::Result<Self, String> {
        let ty = Ty::from_str(s)?;
        Ok(Type::from(ty, gen))
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        fn id_to_name(mut id: Id) -> String {
            let mut name = String::new();
            while id != 0 {
                id -= 1;
                name.insert(0, ('a' as u8 + (id % 26) as u8) as char);
                id /= 26;
            }
            name
        }

        fn to_string(t: &Type, generics: &mut HashMap<Id, String>) -> String {
            match t {
                Type::Const(name) => name.clone(),
                Type::App(t, params) => {
                    let mut string = format!("{}[", to_string(t, generics));
                    if !params.is_empty() {
                        string += &to_string(&params[0], generics);
                    }
                    for i in 1..params.len() {
                        string += &format!(", {}", to_string(&params[i], generics));
                    }
                    string + "]"
                }
                Type::Arrow(init, tail) => {
                    let mut string: String;
                    match init.len() {
                        0 => {
                            string = format!("() -> ");
                        }
                        1 => match init[0] {
                            Type::Const(_)
                            | Type::App(_, _)
                            | Type::TypeVar(_, _)
                            | Type::Generic(_) => {
                                string = format!("{} -> ", to_string(&init[0], generics));
                            }
                            _ => {
                                string = format!("({}) -> ", to_string(&init[0], generics));
                            }
                        },
                        len @ _ => {
                            string = format!("({}", to_string(&init[0], generics));
                            for i in 1..len {
                                string += &format!(", {}", to_string(&init[i], generics));
                            }
                            string.push_str(") -> ");
                        }
                    };
                    string + &format!("{}", to_string(tail, generics))
                }
                Type::TypeVar(id, level) => format!("[{},{}]", id, level),
                Type::Generic(id) => {
                    if let Some(name) = generics.get(id) {
                        format!("{}", name)
                    } else {
                        let name = id_to_name(generics.len() + 1);
                        generics.insert(*id, name.clone());
                        name
                    }
                }
            }
        }

        let mut generics = HashMap::new();
        let string = to_string(self, &mut generics);
        if !generics.is_empty() {
            let mut generics: Vec<&String> = generics.values().collect();
            generics.sort();
            let mut generics = generics.iter();
            write!(fmt, "forall[{}", generics.next().unwrap())?;
            for name in generics {
                write!(fmt, ", {}", name)?;
            }
            write!(fmt, "] ")?;
        }
        write!(fmt, "{}", string)
    }
}
