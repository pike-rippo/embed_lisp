use std::collections::HashMap;

use parking_lot::RwLock;

use crate::{
    builtins::register_all,
    error::{Result, SyntaxError},
    expression::Exp,
    typedef::{Shared, SharedEnv},
};

#[derive(Debug)]
pub struct Env {
    current: RwLock<HashMap<String, Exp>>,
    parent: Option<SharedEnv>,
    level: u8,
}

impl Env {
    pub fn new() -> SharedEnv {
        Self::new_with_builtin(Self::builtin_env())
    }

    pub fn new_with_builtin(builtin: SharedEnv) -> SharedEnv {
        Self::new_child(builtin)
    }

    pub fn builtin_env() -> SharedEnv {
        let env = Shared::new(Self {
            current: RwLock::new(HashMap::new()),
            parent: None,
            level: 0,
        });

        register_all(&env);
        env
    }

    pub fn new_child(parent: SharedEnv) -> SharedEnv {
        let level = parent.level + 1;
        Shared::new(Self {
            current: RwLock::new(HashMap::new()),
            parent: Some(parent),
            level,
        })
    }

    pub fn extend(parent: SharedEnv, keys: &[String], values: &[Exp]) -> SharedEnv {
        let child = Env::new_child(parent);
        {
            let mut current = child.current.write();
            for (key, value) in keys.iter().zip(values.iter()) {
                current.insert(key.clone(), value.clone());
            }
        }
        child
    }

    pub fn extend_dotted(parent: SharedEnv, keys: &[String], values: &[Exp]) -> SharedEnv {
        let child = Env::new_child(parent);
        let (tail, keys) = keys.split_last().expect("extend dotted");
        let (pairs, _, values) = zip_with_remainder_iter(keys.iter(), values.iter());
        {
            let mut current = child.current.write();
            for (key, value) in pairs {
                current.insert(key.clone(), value.clone());
            }
            current.insert(
                tail.clone(),
                Exp::List(values.cloned().collect::<Vec<Exp>>()),
            );
        }
        child
    }

    #[cfg(feature = "async")]
    pub fn deep_copy(&self) -> SharedEnv {
        Shared::new(Self {
            current: RwLock::new(self.current.read().clone()),
            parent: self.parent.as_ref().map(|outer_env| outer_env.deep_copy()),
            level: self.level,
        })
    }

    pub fn define(&self, k: &str, v: Exp) -> Exp {
        self.current.write().insert(k.to_string(), v.clone());

        v
    }

    pub fn assign(&self, k: &str, v: Exp) -> Exp {
        if k.starts_with('*') && k.ends_with('*') && k.len() > 1 {
            if self.level == 1 {
                return self.define(k, v);
            } else {
                let global = self.find_env_by_level(1).unwrap();
                return global.define(k, v);
            }
        }

        if self.current.read().contains_key(k) {
            return self.define(k, v);
        }

        if let Some(parent) = self.find_env_by_key(k) {
            parent.define(k, v)
        } else {
            Exp::Nil
        }
    }

    pub fn drop_symbol(&self, l: &str) -> Exp {
        self.current.write().remove(l).unwrap_or(Exp::Nil)
    }

    pub fn find_env_by_level(&self, level: u8) -> Option<SharedEnv> {
        match &self.parent {
            None => None,
            Some(parent) => {
                if parent.level == level {
                    Some(parent.clone())
                } else {
                    parent.find_env_by_level(level)
                }
            }
        }
    }

    pub fn find_env_by_key(&self, k: &str) -> Option<SharedEnv> {
        match &self.parent {
            None => None,
            Some(parent) => {
                // let found = parent.find_env_by_key(k);
                // found.or_else(|| Some(parent.clone()))
                if parent.current.read().contains_key(k) {
                    Some(parent.clone())
                } else {
                    parent.find_env_by_key(k)
                }
            }
        }
    }

    pub fn try_lookup(&self, k: &str) -> Result<Exp> {
        self.lookup(k).ok_or(SyntaxError::unbound_symbol(k))
    }

    pub fn lookup(&self, k: &str) -> Option<Exp> {
        if self.current.read().contains_key(k) {
            return self.current.read().get(k).cloned();
        }

        if let Some(e) = self.lookup_parent(k) {
            return Some(e);
        }

        None
    }

    pub fn lookup_parent(&self, k: &str) -> Option<Exp> {
        match self.parent {
            None => None,
            Some(ref parent) => {
                if parent.current.read().contains_key(k) {
                    return parent.current.read().get(k).cloned();
                }

                parent.lookup_parent(k)
            }
        }
    }

    pub fn dump(&self, show_builtin: bool) {
        if !show_builtin && self.level == 0 {
            return;
        }
        let level = if self.level != 0 {
            &self.level.to_string()[..]
        } else {
            "Builtin"
        };
        println!("Level: {}", level);

        for (k, v) in self.current.read().iter() {
            println!("   '{}' = {}", k, v);
        }

        if let Some(parent) = &self.parent {
            parent.dump(show_builtin);
        }
    }
}

fn zip_with_remainder_iter<'a, A, B, I1, I2>(
    mut iter1: I1,
    mut iter2: I2,
) -> (
    Vec<(A, B)>,
    Box<dyn Iterator<Item = A> + 'a>,
    Box<dyn Iterator<Item = B> + 'a>,
)
where
    A: 'a,
    B: 'a,
    I1: Iterator<Item = A> + 'a,
    I2: Iterator<Item = B> + 'a,
{
    let mut zipped = Vec::new();

    loop {
        match (iter1.next(), iter2.next()) {
            (Some(a), Some(b)) => zipped.push((a, b)),
            (Some(a), None) => {
                return (
                    zipped,
                    Box::new(std::iter::once(a).chain(iter1)),
                    Box::new(std::iter::empty()),
                );
            }
            (None, Some(b)) => {
                return (
                    zipped,
                    Box::new(std::iter::empty()),
                    Box::new(std::iter::once(b).chain(iter2)),
                );
            }
            (None, None) => {
                return (
                    zipped,
                    Box::new(std::iter::empty()),
                    Box::new(std::iter::empty()),
                );
            }
        }
    }
}
