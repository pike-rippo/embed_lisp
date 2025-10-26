use std::collections::HashMap;

use parking_lot::RwLock;

use crate::{
    Error,
    builtins::register_all,
    error::Result,
    expression::Exp,
    typedef::{Shared, SharedEnv},
};

#[derive(Debug)]
pub struct Env {
    current: RwLock<HashMap<String, Exp>>,
    outer: Option<SharedEnv>,
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
            outer: None,
            level: 0,
        });

        register_all(&env);
        env
    }

    pub fn new_child(parent: SharedEnv) -> SharedEnv {
        let level = parent.level + 1;
        Shared::new(Self {
            current: RwLock::new(HashMap::new()),
            outer: Some(parent),
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
            outer: self.outer.as_ref().map(|outer_env| outer_env.deep_copy()),
            level: self.level,
        })
    }

    pub fn define(&self, k: &str, v: Exp) -> Exp {
        self.current.write().insert(k.to_string(), v.clone());

        v
    }

    pub fn assign(&self, k: &str, v: Exp) -> Exp {
        if self.current.read().contains_key(k) {
            return self.define(k, v);
        }

        if let Some(outer) = self.find_env(k) {
            outer.define(k, v)
        } else {
            Exp::Nil
        }
    }

    pub fn find_env(&self, k: &str) -> Option<SharedEnv> {
        if self.current.read().contains_key(k) {
            return None;
        }

        match &self.outer {
            None => None,
            Some(outer) => {
                let found = outer.find_env(k);
                found.or_else(|| Some(outer.clone()))
            }
        }
    }

    pub fn try_lookup(&self, k: &str) -> Result<Exp> {
        self.lookup(k)
            .ok_or(Error::Reason(format!("unexpected symbol '{}'", k)))
    }

    pub fn lookup(&self, k: &str) -> Option<Exp> {
        if self.current.read().contains_key(k) {
            return self.current.read().get(k).cloned();
        }

        if let Some(e) = self.lookup_outer(k) {
            return Some(e);
        }

        None
    }

    pub fn lookup_outer(&self, k: &str) -> Option<Exp> {
        match self.outer {
            None => None,
            Some(ref outer) => {
                if outer.current.read().contains_key(k) {
                    return outer.current.read().get(k).cloned();
                }
                // if outer.current.blocking_lock().contains_key(k) {
                //     return outer.current.blocking_lock().get(k).cloned();
                // }

                outer.lookup_outer(k)
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

        if let Some(outer) = &self.outer {
            outer.dump(show_builtin);
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
