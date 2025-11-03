use std::{collections::HashMap, fmt::Display};

use parking_lot::RwLock;

use crate::{
    Evaluator, error::SyntaxError, expression::Exp, flow::EvalResult, native::NativeObject,
    typedef::Shared,
};

pub struct HashMapObject {
    inner: RwLock<HashMap<Exp, Exp>>,
}

pub fn register(eval: &Evaluator) {
    eval.register_native_object_creator("HashMap", |_args: &[Exp]| {
        Ok(Exp::Native(Shared::new(HashMapObject::new())).value_flow())
    });
}

impl HashMapObject {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    fn handle_insert(&self, args: &[Exp]) -> EvalResult {
        if args.len() != 2 {
            return Err(SyntaxError::invalid_args_size("insert", 2, args.len()));
        }

        let (k, v) = (&args[0], &args[1]);
        k.check_key_allowed()?;

        self.inner.write().insert(k.clone(), v.clone());

        Ok(Exp::Bool(true).value_flow())
    }

    fn handle_get(&self, args: &[Exp]) -> EvalResult {
        if args.len() != 1 {
            return Err(SyntaxError::invalid_args_size("get", 1, args.len()));
        }

        let k = &args[0];
        k.check_key_allowed()?;

        Ok(self
            .inner
            .read()
            .get(k)
            .cloned()
            .unwrap_or(Exp::Nil)
            .value_flow())
    }

    fn handle_remove(&self, args: &[Exp]) -> EvalResult {
        if args.len() != 1 {
            return Err(SyntaxError::invalid_args_size("remove", 1, args.len()));
        }

        let k = &args[0];
        k.check_key_allowed()?;

        Ok(self
            .inner
            .write()
            .remove(k)
            .unwrap_or(Exp::Nil)
            .value_flow())
    }

    fn handle_keys(&self, _args: &[Exp]) -> EvalResult {
        let keys = self
            .inner
            .read()
            .keys()
            .map(|e| e.as_string())
            .collect::<Vec<String>>();
        Ok(Exp::List(keys.into_iter().map(Exp::String).collect()).value_flow())
    }

    fn handle_len(&self, _args: &[Exp]) -> EvalResult {
        Ok(Exp::Number(self.inner.read().len() as f64).value_flow())
    }
}

impl Display for HashMapObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HashMap")
    }
}

impl NativeObject for HashMapObject {
    fn call_method(&self, name: &str, args: &[Exp]) -> EvalResult {
        match name {
            "insert" => self.handle_insert(args),
            "get" => self.handle_get(args),
            "remove" => self.handle_remove(args),
            "keys" => self.handle_keys(args),
            "len" => self.handle_len(args),
            _ => Err(SyntaxError::no_such_method("HashMap", name)),
        }
    }
}
