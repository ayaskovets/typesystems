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

    pub fn lookup_binding(&self, mut id: Id, mut level: Level) -> Option<&T> {
        let mut prev: Option<&T> = None;
        loop {
            if let Some(next) = self.bound.get(&(id, level)) {
                prev = Some(next);
                if let Some((id2, level2)) = Bindable::get_unbound_id_level(next) {
                    id = id2;
                    level = level2;
                    continue;
                }
            }
            break;
        }
        prev
    }
}
