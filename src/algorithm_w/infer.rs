/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::{HashMap, HashSet};

use crate::{Env, Gen, Id, Level, Term, Type};

fn instantiate(t: &Type, level: Level, gen: &mut Gen<Type>) -> Type {
    struct Instantiate<'a> {
        instantiated: HashMap<Id, Type>,
        gen: &'a mut Gen<Type>,
    }

    impl<'a> Instantiate<'a> {
        pub fn new(gen: &'a mut Gen<Type>) -> Self {
            Instantiate {
                instantiated: HashMap::new(),
                gen,
            }
        }

        pub fn instantiate(&mut self, t: &Type, level: Level) -> Type {
            match t {
                Type::Const(_) | Type::Unbound(_, _) => t.clone(),
                Type::App(t, params) => Type::App(
                    Box::new(self.instantiate(t, level)),
                    params
                        .iter()
                        .map(|t_param| self.instantiate(t_param, level))
                        .collect(),
                ),
                Type::Arrow(init, tail) => Type::Arrow(
                    init.iter()
                        .map(|t_param| self.instantiate(t_param, level))
                        .collect(),
                    Box::new(self.instantiate(tail, level)),
                ),
                Type::Generic(id) => {
                    if let Some(t) = self.instantiated.get(&id) {
                        t.clone()
                    } else {
                        let var = self.gen.newvar(Some(level));
                        self.instantiated.insert(*id, var.clone());
                        var
                    }
                }
                Type::Alias(t) => self.instantiate(t, level),
            }
        }
    }

    Instantiate::new(gen).instantiate(t, level)
}

pub fn generalize(t: &Type, level: Level) -> Type {
    match t {
        Type::Unbound(id, level2) if *level2 > level => Type::Generic(*id),
        Type::Const(_) | Type::Generic(_) | Type::Unbound(_, _) => t.clone(),
        Type::App(t, params) => Type::App(
            Box::new(generalize(t, level)),
            params
                .iter()
                .map(|t_param| generalize(t_param, level))
                .collect(),
        ),
        Type::Arrow(init, tail) => Type::Arrow(
            init.iter()
                .map(|t_param| generalize(t_param, level))
                .collect(),
            Box::new(generalize(tail, level)),
        ),
        Type::Alias(t) => generalize(t, level),
    }
}

pub fn infer(term: &Term, env: &Env<Type>, gen: &Gen<Type>) -> Result<Type, String> {
    struct Infer {
        env: Env<Type>,
        gen: Gen<Type>,
    }

    impl Infer {
        pub fn new(env: Env<Type>, gen: Gen<Type>) -> Self {
            Infer { env, gen }
        }

        pub fn infer(&mut self, term: &Term, level: Level) -> Result<Type, String> {
            match term {
                Term::Var(name) => {
                    if let Some(t) = self.env.lookup(name) {
                        Ok(instantiate(&t, level, &mut self.gen))
                    } else {
                        Err(format!("Undefined variable '{}'", name))
                    }
                }
                Term::Let(name, assign, body) => {
                    let t_assign = self.infer(assign, level + 1)?;

                    let old = self.env.insert(name, t_assign);
                    let t_body = self.infer(body, level)?;
                    self.env.remove(name);

                    if let Some(t_old) = old {
                        self.env.insert(name, t_old);
                    }
                    Ok(t_body)
                }
                Term::Abs(args, body) => {
                    let mut unique = HashSet::new();
                    for arg in args {
                        if !unique.insert(arg) {
                            return Err(format!(
                                "Conflicting definitions of {} in \\{} -> {}",
                                arg,
                                args.join(" "),
                                body
                            ));
                        }
                    }

                    let mut old = HashMap::new();
                    let t_args: Vec<Type> = args
                        .iter()
                        .map(|arg| {
                            let t_arg = self.gen.newvar(Some(level));
                            if let Some(t_old) = self.env.insert(arg, t_arg.clone()) {
                                old.insert(arg, t_old);
                            }
                            t_arg
                        })
                        .collect();
                    let t_body = self.infer(body, level)?;
                    for arg in args {
                        if let Some(t_old) = old.remove(arg) {
                            self.env.insert(arg, t_old);
                        }
                    }

                    Ok(Type::Arrow(t_args, Box::new(t_body)))
                }
                Term::App(f, args) => {

                    todo!()
                }
            }
        }
    }

    Infer::new(env.clone(), gen.clone()).infer(term, 0)
}
