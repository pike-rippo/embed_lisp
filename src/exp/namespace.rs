use std::collections::HashMap;

use parking_lot::RwLock;

use crate::{
    Exp,
    environment::SharedEnv,
    error::{Result, SyntaxError},
    visit::HashMapIter,
};

pub struct NameSpace {
    inner: RwLock<HashMap<String, Exp>>,
}

impl Default for NameSpace {
    fn default() -> Self {
        Self {
            inner: RwLock::new(HashMap::default()),
        }
    }
}

impl NameSpace {
    pub fn new(binding: &[(&str, Exp)]) -> Self {
        let ns = Self::default();
        binding.iter().for_each(|(k, v)| {
            ns.define(k, v.clone());
        });
        ns
    }

    pub fn define(&self, k: &str, v: Exp) -> Exp {
        self.inner.write().insert(k.to_string(), v.clone());
        v
    }

    pub fn try_lookup(&self, k: &str) -> Result<Exp> {
        self.lookup(k).ok_or(SyntaxError::unbound_symbol(k))
    }

    pub fn lookup(&self, k: &str) -> Option<Exp> {
        self.inner.read().get(k).cloned()
    }

    pub fn expand(&self, env: &SharedEnv) {
        for (k, v) in self.inner.read().iter() {
            env.define(k, v.clone());
        }
    }

    pub fn current_iter(&self) -> HashMapIter<'_> {
        let guard = self.inner.read();
        let iter = unsafe {
            std::mem::transmute::<_, std::collections::hash_map::Iter<'_, String, Exp>>(
                guard.iter(),
            )
        };
        HashMapIter::new(guard, iter)
    }
}
