/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;

pub type Id = usize;
pub type Level = isize;

pub trait Bindable {
    fn get_unbound_id_level(_: &Self) -> Option<(Id, Level)>;
}

#[derive(Clone)]
pub struct Gen<T>
where
    T: From<(Id, Option<Level>)>,
{
    id: Id,
    t: std::marker::PhantomData<T>,
}

impl<T> Gen<T>
where
    T: From<(Id, Option<Level>)>,
{
    pub fn new() -> Self {
        Gen {
            id: 1,
            t: std::marker::PhantomData,
        }
    }

    pub fn newvar(&mut self, level: Option<Level>) -> T {
        let next_id = self.id;
        self.id += 1;
        T::from((next_id, level))
    }

    pub fn reset(&mut self) {
        self.id = 1;
    }
}

#[derive(Clone)]
pub struct Env<T>
where
    T: From<(Id, Option<Level>)> + Bindable,
{
    env: HashMap<String, T>,
    bound: HashMap<(Id, Level), T>,
}

impl<T> Env<T>
where
    T: From<(Id, Option<Level>)> + Bindable + std::fmt::Display,
{
    pub fn new() -> Self {
        Env {
            env: HashMap::new(),
            bound: HashMap::new(),
        }
    }

    pub fn insert(&mut self, k: &str, v: T) -> Option<T> {
        self.env.insert(k.to_owned(), v.into())
    }

    pub fn remove(&mut self, k: &str) -> Option<T> {
        self.env.remove(k)
    }

    pub fn lookup(&self, k: &str) -> Option<&T> {
        self.env.get(k)
    }

    pub fn bind(&mut self, id: Id, level: Level, v: T) -> Option<T> {
        self.bound.insert((id, level), v)
    }

    pub fn unbind(&mut self, id: Id, level: Level) -> Option<T> {
        self.bound.remove(&(id, level))
    }

    pub fn lookup_binding(&self, id: Id, level: Level) -> Option<&T> {
        if let Some(v) = self.bound.get(&(id, level)) {
            if let Some((id, level)) = Bindable::get_unbound_id_level(v) {
                if let Some(binding) = self.lookup_binding(id, level) {
                    return Some(binding);
                }
            }
            return Some(v);
        }
        None
    }
}
