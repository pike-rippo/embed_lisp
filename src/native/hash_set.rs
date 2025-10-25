use std::collections::HashSet;

use parking_lot::RwLock;

use crate::{Evaluator, Exp, NativeObject, error::Result, ok, typedef::Shared};

pub struct HashSetObject {
    inner: RwLock<HashSet<Exp>>,
}

pub fn register(eval: &Evaluator) {
    eval.register_native_object_creator("HashSet", |_args: &[Exp]| {
        Ok(Exp::Native(Shared::new(HashSetObject::new())))
    });
}

impl HashSetObject {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashSet::new()),
        }
    }

    fn handle_insert(&self, args: &[Exp]) -> Result<Exp> {
        if args.len() != 1 {
            return Err("insert expected 1 argument".into());
        }

        let value = &args[0];
        value.check_key_allowed()?;

        ok!(self.inner.write().insert(value.clone()))
    }

    fn handle_remove(&self, args: &[Exp]) -> Result<Exp> {
        if args.len() != 1 {
            return Err("remove expected 1 argument".into());
        }

        let value = &args[0];
        value.check_key_allowed()?;

        ok!(self.inner.write().remove(value))
    }

    fn handle_contains(&self, args: &[Exp]) -> Result<Exp> {
        if args.len() != 1 {
            return Err("contains expected 1 argument".into());
        }

        let value = &args[0];
        value.check_key_allowed()?;

        ok!(self.inner.write().contains(value))
    }

    fn handle_clear(&self, args: &[Exp]) -> Result<Exp> {
        if args.len() != 0 {
            return Err("clear expected 0 argument".into());
        }

        self.inner.write().clear();
        ok!(true)
    }

    fn handle_len(&self, args: &[Exp]) -> Result<Exp> {
        if args.len() != 0 {
            return Err("len expected 0 argument".into());
        }

        ok!(self.inner.write().len() as f64)
    }
}

impl NativeObject for HashSetObject {
    fn get_type_name(&self) -> &'static str {
        "HashSet"
    }

    fn call_method(&self, name: &str, args: &[Exp]) -> crate::error::Result<Exp> {
        match name {
            "insert" => self.handle_insert(args),
            "remove" => self.handle_remove(args),
            "contains" => self.handle_contains(args),
            "clear" => self.handle_clear(args),
            "len" => self.handle_len(args),
            _ => Err(format!("HashSet has no method {}", name).into()),
        }
    }
}
