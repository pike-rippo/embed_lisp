use std::collections::HashSet;

use parking_lot::RwLock;

use crate::{Evaluator, Exp, NativeObject, flow::EvalResult, typedef::Shared};

pub struct HashSetObject {
    inner: RwLock<HashSet<Exp>>,
}

pub fn register(eval: &Evaluator) {
    eval.register_native_object_creator("HashSet", |_args: &[Exp]| {
        Ok(Exp::Native(Shared::new(HashSetObject::new())).value_flow())
    });
}

impl HashSetObject {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashSet::new()),
        }
    }

    fn handle_insert(&self, args: &[Exp]) -> EvalResult {
        if args.len() != 1 {
            return Err("insert expected 1 argument".into());
        }

        let value = &args[0];
        value.check_key_allowed()?;

        Ok(Exp::Bool(self.inner.write().insert(value.clone())).value_flow())
    }

    fn handle_remove(&self, args: &[Exp]) -> EvalResult {
        if args.len() != 1 {
            return Err("remove expected 1 argument".into());
        }

        let value = &args[0];
        value.check_key_allowed()?;

        Ok(Exp::Bool(self.inner.write().remove(value)).value_flow())
    }

    fn handle_contains(&self, args: &[Exp]) -> EvalResult {
        if args.len() != 1 {
            return Err("contains expected 1 argument".into());
        }

        let value = &args[0];
        value.check_key_allowed()?;

        Ok(Exp::Bool(self.inner.write().contains(value)).value_flow())
    }

    fn handle_clear(&self, args: &[Exp]) -> EvalResult {
        if args.is_empty() {
            return Err("clear expected 0 argument".into());
        }

        self.inner.write().clear();
        Ok(Exp::Bool(true).value_flow())
    }

    fn handle_len(&self, args: &[Exp]) -> EvalResult {
        if args.is_empty() {
            return Err("len expected 0 argument".into());
        }

        Ok(Exp::Number(self.inner.write().len() as f64).value_flow())
    }
}

impl NativeObject for HashSetObject {
    fn get_type_name(&self) -> &'static str {
        "HashSet"
    }

    fn call_method(&self, name: &str, args: &[Exp]) -> EvalResult {
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
