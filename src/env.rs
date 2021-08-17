/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;

pub type Id = usize;
pub type Level = usize;

pub struct Env<Ty>
where
    Ty: From<(Id, Option<Level>)>,
{
    env: HashMap<String, Ty>,
    id: Id,
}

impl<Ty> Env<Ty>
where
    Ty: From<(Id, Option<Level>)>,
{
    pub fn new() -> Self {
        Env {
            env: HashMap::new(),
            id: 0,
        }
    }

    pub fn insert(&mut self, k: &str, v: Ty) {
        self.env.insert(k.to_owned(), v.into());
    }
    pub fn remove(&mut self, k: &str) {
        self.env.remove(k);
    }
    pub fn lookup(&self, k: &str) -> Option<&Ty> {
        self.env.get(k)
    }
    pub fn lookup_mut(&mut self, k: &str) -> Option<&mut Ty> {
        self.env.get_mut(k)
    }
    pub fn clear(&mut self) {
        self.env.clear();
    }

    pub fn newvar(&mut self, level: Option<Level>) -> Ty {
        let next_id = self.id;
        self.id += 1;
        Ty::from((next_id, level))
    }
}
