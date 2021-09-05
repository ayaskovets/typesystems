/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::{HashMap, HashSet};

use crate::{Env, Gen, Id, Level, Term, Type};

fn unify(t1: &Type, t2: &Type) -> Result<Type, String> {
    if *t1 == *t2 {
        return Ok(t1.clone());
    }

    match (t1, t2) {
        (Type::Const(name1), Type::Const(name2)) if name1 == name2 => {
            Ok(Type::Const(name1.to_string()))
        }
        (Type::App(t1, params1), Type::App(t2, params2)) => {
            let t = unify(t1, t2)?;
            let mut params = Vec::new();
            for (param1, param2) in params1.iter().zip(params2) {
                params.push(unify(param1, param2)?);
            }
            Ok(Type::App(Box::new(t), params))
        }
        (Type::Arrow(init1, tail1), Type::Arrow(init2, tail2)) => {
            let mut init = Vec::new();
            for (param1, param2) in init1.iter().zip(init2) {
                init.push(unify(param1, param2)?);
            }
            Ok(Type::Arrow(init, Box::new(unify(tail1, tail2)?)))
        }
        (Type::Unbound(id1, _), Type::Unbound(id2, _)) if id1 == id2 => {
            Err(format!("Multiple instances of variable with id {}", id1))
        }
        (Type::Unbound(id, level), t) | (t, Type::Unbound(id, level)) => Ok(t.clone()),
        (t1, t2) => Err(format!("Cannot unify {} with {}", t1, t2)),
    }
}

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
                        Ok(instantiate(t, level, &mut self.gen))
                    } else {
                        Err(format!("Undefined variable '{}'", name))
                    }
                }
                Term::Let(name, assign, body) => {
                    let t_assign = generalize(&self.infer(assign, level + 1)?, level);

                    let shadowing = self.env.insert(name, t_assign);
                    let t_body = self.infer(body, level)?;
                    self.env.remove(name);

                    if let Some(t_old) = shadowing {
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

                    let mut shadowing = HashMap::new();
                    let t_args = args
                        .iter()
                        .map(|arg| {
                            let t_arg = self.gen.newvar(Some(level));
                            if let Some(t_old) = self.env.insert(arg, t_arg.clone()) {
                                shadowing.insert(arg, t_old);
                            }
                            t_arg
                        })
                        .collect();
                    let t_body = self.infer(body, level)?;
                    for arg in args {
                        if let Some(t_old) = shadowing.remove(arg) {
                            self.env.insert(arg, t_old);
                        }
                    }

                    Ok(Type::Arrow(t_args, Box::new(t_body)))
                }
                Term::App(f, args) => {
                    let (t_args, t_return) = match self.infer(f, level)? {
                        Type::Arrow(init, tail) => {
                            if init.len() == args.len() {
                                Ok((init, *tail))
                            } else {
                                Err(format!(
                                    "Incorrect number of arguments in call to {}. Must be {}",
                                    f,
                                    args.len()
                                ))
                            }
                        }
                        Type::Unbound(_, level) => {
                            let init: Vec<Type> =
                                std::iter::repeat_with(|| self.gen.newvar(Some(level)))
                                    .take(args.len())
                                    .collect();
                            let tail = self.gen.newvar(Some(level));
                            Ok((init, tail))
                        }
                        t_f @ _ => Err(format!("Invalid type of function: {}", t_f)),
                    }?;

                    for (arg, t_arg) in args.iter().zip(t_args.iter()) {
                        unify(&t_arg, &self.infer(arg, level)?)?;
                    }

                    Ok(t_return)
                }
            }
        }
    }

    Infer::new(env.clone(), gen.clone()).infer(term, 0)
}
