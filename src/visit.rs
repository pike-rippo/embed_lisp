use std::collections::HashMap;

use crate::{Exp, environment::Env, exp::NameSpace};

mod dump;
mod full_dump;

pub use dump::DumpVisitor;
pub use full_dump::FullDumpVisitor;

pub trait Visitor {
    fn result(&self) -> &str;
    fn visit_env(&mut self, env: &Env);
    fn visit_exp(&mut self, exp: &Exp);
    fn visit_ns(&mut self, ns: &NameSpace);
}

pub trait Visitable {
    fn accept(&self, visitor: &mut dyn Visitor);
}

pub struct HashMapIter<'a> {
    _guard: parking_lot::RwLockReadGuard<'a, HashMap<String, Exp>>,
    iter: std::collections::hash_map::Iter<'a, String, Exp>,
}

impl<'a> HashMapIter<'a> {
    pub fn new(
        _guard: parking_lot::RwLockReadGuard<'a, HashMap<String, Exp>>,
        iter: std::collections::hash_map::Iter<'a, String, Exp>,
    ) -> Self {
        Self { _guard, iter }
    }
}

impl<'a> Iterator for HashMapIter<'a> {
    type Item = (&'a String, &'a Exp);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
