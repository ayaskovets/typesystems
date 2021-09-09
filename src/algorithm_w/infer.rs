/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::{HashMap, HashSet};

use crate::{Env, Gen, Id, Level, Term, Type};

pub fn unify(t1: &Type, t2: &Type, env: &mut Env<Type>) -> Result<Type, String> {
    struct Unify<'a> {
        env: &'a mut Env<Type>,
    }

    impl<'a> Unify<'a> {
        pub fn new(env: &'a mut Env<Type>) -> Self {
            Unify { env }
        }

        fn update_bound_levels(&mut self, id: Id, level: Level, t: &Type) -> Result<(), String> {
            match t {
                Type::TypeVar(id2, level2) => {
                    if id == *id2 {
                        Err(format!("Infinite type"))
                    } else if *level2 > level {
                        self.env.bind(*id2, *level2, Type::TypeVar(*id2, level));
                        Ok(())
                    } else {
                        Ok(())
                    }
                }
                Type::Const(_) => Ok(()),
                Type::Generic(_) => unreachable!(),
                Type::App(t, params) => {
                    self.update_bound_levels(id, level, t)?;
                    for param in params {
                        self.update_bound_levels(id, level, param)?;
                    }
                    Ok(())
                }
                Type::Arrow(init, tail) => {
                    for param in init {
                        self.update_bound_levels(id, level, param)?;
                    }
                    self.update_bound_levels(id, level, tail)
                }
            }
        }

        pub fn unify(&mut self, t1: &Type, t2: &Type) -> Result<Type, String> {
            if *t1 == *t2 {
                return Ok(t1.clone());
            }

            match (t1, t2) {
                (Type::Const(name1), Type::Const(name2)) if name1 == name2 => {
                    Ok(Type::Const(name1.to_string()))
                }
                (Type::App(t1, params1), Type::App(t2, params2)) => {
                    let t = self.unify(t1, t2)?;
                    let mut params = Vec::new();
                    for (param1, param2) in params1.iter().zip(params2) {
                        params.push(self.unify(param1, param2)?);
                    }
                    Ok(Type::App(Box::new(t), params))
                }
                (Type::Arrow(init1, tail1), Type::Arrow(init2, tail2)) => {
                    let mut init = Vec::new();
                    for (param1, param2) in init1.iter().zip(init2) {
                        init.push(self.unify(param1, param2)?);
                    }
                    Ok(Type::Arrow(init, Box::new(self.unify(tail1, tail2)?)))
                }
                (Type::TypeVar(id1, _), Type::TypeVar(id2, _)) if id1 == id2 => {
                    Err(format!("Multiple instances of variable"))
                }
                (Type::TypeVar(id, level), t) | (t, Type::TypeVar(id, level)) => {
                    if let Some(binding) = self.env.lookup_binding(*id, *level) {
                        let binding = binding.clone();
                        self.unify(&binding, t)
                    } else {
                        self.env.bind(*id, *level, t.clone());
                        self.update_bound_levels(*id, *level, t)?;
                        Ok(t.clone())
                    }
                }
                (t1, t2) => Err(format!("Cannot unify {} with {}", t1, t2)),
            }
        }
    }
    Unify::new(env).unify(t1, t2)
}

pub fn instantiate(t: &Type, level: Level, gen: &mut Gen<Type>, env: &Env<Type>) -> Type {
    struct Instantiate<'a, 'b> {
        instantiated: HashMap<Id, Type>,
        gen: &'a mut Gen<Type>,
        env: &'b Env<Type>,
    }

    impl<'a, 'b> Instantiate<'a, 'b> {
        pub fn new(gen: &'a mut Gen<Type>, env: &'b Env<Type>) -> Self {
            Instantiate {
                instantiated: HashMap::new(),
                gen,
                env,
            }
        }

        pub fn instantiate(&mut self, t: &Type, level: Level) -> Type {
            match t {
                Type::TypeVar(id, level) => {
                    if let Some(binding) = self.env.lookup_binding(*id, *level) {
                        self.instantiate(binding, *level)
                    } else {
                        Type::TypeVar(*id, *level)
                    }
                }
                Type::Const(_) => t.clone(),
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

    Instantiate::new(gen, env).instantiate(t, level)
}

pub fn generalize(t: &Type, level: Level, env: &Env<Type>) -> Type {
    struct Generalize<'a> {
        env: &'a Env<Type>,
    }

    impl<'a> Generalize<'a> {
        pub fn new(env: &'a Env<Type>) -> Self {
            Generalize { env }
        }

        pub fn generalize(&self, t: &Type, level: Level) -> Type {
            match t {
                Type::TypeVar(id, level2) => {
                    if let Some(binding) = self.env.lookup_binding(*id, *level2) {
                        self.generalize(binding, level)
                    } else if *level2 > level {
                        Type::Generic(*id)
                    } else {
                        Type::TypeVar(*id, *level2)
                    }
                }
                Type::Const(_) | Type::Generic(_) => t.clone(),
                Type::App(t, params) => Type::App(
                    Box::new(self.generalize(t, level)),
                    params
                        .iter()
                        .map(|t_param| self.generalize(t_param, level))
                        .collect(),
                ),
                Type::Arrow(init, tail) => Type::Arrow(
                    init.iter()
                        .map(|t_param| self.generalize(t_param, level))
                        .collect(),
                    Box::new(self.generalize(tail, level)),
                ),
            }
        }
    }

    Generalize::new(env).generalize(t, level)
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

        fn genarrow(&mut self, t: Type, args_len: usize) -> Result<(Vec<Type>, Type), String> {
            match t {
                Type::Arrow(init, tail) => {
                    if init.len() == args_len {
                        Ok((init.clone(), *tail.clone()))
                    } else {
                        Err(format!(
                            "Incorrect number of arguments. Must be {}",
                            args_len
                        ))
                    }
                }
                Type::TypeVar(id, level) => {
                    if let Some(binding) = self.env.lookup_binding(id, level) {
                        let binding = binding.clone();
                        return self.genarrow(binding, args_len);
                    }

                    let init: Vec<Type> = std::iter::repeat_with(|| self.gen.newvar(Some(level)))
                        .take(args_len)
                        .collect();
                    let tail = self.gen.newvar(Some(level));
                    self.env
                        .bind(id, level, Type::Arrow(init.clone(), Box::new(tail.clone())));
                    Ok((init, tail))
                }
                t_f @ _ => Err(format!("Invalid type of function: {}", t_f)),
            }
        }

        pub fn infer(&mut self, term: &Term, level: Level) -> Result<Type, String> {
            match term {
                Term::Var(name) => {
                    if let Some(t) = self.env.lookup(name) {
                        Ok(instantiate(t, level, &mut self.gen, &self.env))
                    } else {
                        Err(format!("Undefined variable '{}'", name))
                    }
                }
                Term::Let(name, assign, body) => {
                    let t_assign = generalize(&self.infer(assign, level + 1)?, level, &self.env);

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
                    let t_args: Vec<Type> = args
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
                    let t_f = self.infer(f, level)?;
                    let (t_args, t_return) = self.genarrow(t_f, args.len())?;

                    for (arg, t_arg) in args.iter().zip(t_args.iter()) {
                        let t_param = self.infer(arg, level)?;
                        unify(&t_arg, &t_param, &mut self.env)?;
                    }

                    Ok(t_return)
                }
            }
        }
    }

    let mut infer = Infer::new(env.clone(), gen.clone());
    let ty = infer.infer(term, 0)?;
    Ok(generalize(&ty, -1, &infer.env))
}
