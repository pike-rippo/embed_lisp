use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{builtins::register_all, expression::Exp};

pub type EnvRc = Rc<Env>;

#[derive(Debug)]
pub struct Env {
    current: RefCell<HashMap<String, Exp>>,
    outer: Option<EnvRc>,
    level: u8,
}

impl Env {
    pub fn new() -> EnvRc {
        Self::new_with_builtin(Self::builtin_env())
    }

    pub fn new_with_builtin(builtin: EnvRc) -> EnvRc {
        Self::new_child(builtin)
    }

    pub fn builtin_env() -> EnvRc {
        let env = Rc::new(Self {
            current: RefCell::new(HashMap::new()),
            outer: None,
            level: 0,
        });

        register_all(&env);
        env
    }

    pub fn new_child(parent: EnvRc) -> EnvRc {
        let level = parent.level + 1;
        Rc::new(Self {
            current: RefCell::new(HashMap::new()),
            outer: Some(parent),
            level,
        })
    }

    pub fn extend(parent: EnvRc, keys: &[String], values: &[Exp]) -> EnvRc {
        let child = Env::new_child(parent);
        {
            let mut current = child.current.borrow_mut();
            for (key, value) in keys.iter().zip(values.iter()) {
                current.insert(key.clone(), value.clone());
            }
        }
        child
    }

    pub fn define(&self, k: &str, v: Exp) -> Exp {
        self.current.borrow_mut().insert(k.to_string(), v.clone());
        v
    }

    pub fn assign(&self, k: &str, v: Exp) -> Exp {
        if self.current.borrow().contains_key(k) {
            self.define(k, v)
        } else if let Some(outer) = self.find_env(k) {
            outer.define(k, v)
        } else {
            Exp::Nil
        }
    }

    pub fn find_env(&self, k: &str) -> Option<EnvRc> {
        if self.current.borrow().contains_key(k) {
            return None;
        }

        match &self.outer {
            None => None,
            Some(outer) => {
                let found = outer.find_env(k);
                found.or_else(|| Some(Rc::clone(outer)))
            }
        }
    }

    pub fn lookup(&self, k: &str) -> Option<Exp> {
        if self.current.borrow().contains_key(k) {
            return self.current.borrow().get(k).cloned();
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
                if outer.current.borrow().contains_key(k) {
                    return outer.current.borrow().get(k).cloned();
                } else {
                    outer.lookup_outer(k)
                }
            }
        }
    }

    pub fn dump(&self) {
        let level = if self.level != 0 {
            &self.level.to_string()[..]
        } else {
            "Builtin"
        };
        println!("Level: {}", level);
        for (k, v) in self.current.borrow().iter() {
            println!("{} = {}", k, v);
        }

        if let Some(outer) = &self.outer {
            outer.dump();
        }
    }
}
