/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;

pub type Id = usize;
pub type Level = isize;

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

pub struct Env<T>
where
    T: From<(Id, Option<Level>)>,
{
    env: HashMap<String, T>,
    pub gen: Gen<T>,
}

impl<T> Env<T>
where
    T: From<(Id, Option<Level>)>,
{
    pub fn new() -> Self {
        Env {
            env: HashMap::new(),
            gen: Gen::new(),
        }
    }

    pub fn insert(&mut self, k: &str, v: T) {
        self.env.insert(k.to_owned(), v.into());
    }

    pub fn remove(&mut self, k: &str) {
        self.env.remove(k);
    }

    pub fn lookup(&self, k: &str) -> Option<&T> {
        self.env.get(k)
    }

    pub fn lookup_mut(&mut self, k: &str) -> Option<&mut T> {
        self.env.get_mut(k)
    }

    pub fn clear(&mut self) {
        self.env.clear();
    }
}
