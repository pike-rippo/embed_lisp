use std::collections::HashMap;

use parking_lot::RwLock;

use crate::{Evaluator, error::Result, expression::Exp, native::NativeObject, ok, typedef::Shared};

pub struct HashMapObject {
    inner: RwLock<HashMap<Exp, Exp>>,
}

pub fn register(eval: &Evaluator) {
    eval.register_native_object_creator("HashMap", |_args: &[Exp]| {
        Ok(Exp::Native(Shared::new(HashMapObject::new())))
    });
}

impl HashMapObject {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    pub fn handle_insert(&self, args: &[Exp]) -> Result<Exp> {
        if args.len() != 2 {
            return Err("insert expected 2 arguments".into());
        }

        let (k, v) = (&args[0], &args[1]);
        k.check_key_allowed()?;

        self.inner.write().insert(k.clone(), v.clone());

        ok!(true)
    }

    pub fn handle_get(&self, args: &[Exp]) -> Result<Exp> {
        if args.len() != 1 {
            return Err("get expected 1 argument".into());
        }

        let k = &args[0];
        k.check_key_allowed()?;

        Ok(self.inner.read().get(k).cloned().unwrap_or(Exp::Nil))
    }

    pub fn handle_remove(&self, args: &[Exp]) -> Result<Exp> {
        if args.len() != 1 {
            return Err("get expected 1 argument".into());
        }

        let k = &args[0];
        k.check_key_allowed()?;

        Ok(self.inner.write().remove(k).unwrap_or(Exp::Nil))
    }

    pub fn handle_keys(&self, _args: &[Exp]) -> Result<Exp> {
        let keys = self
            .inner
            .read()
            .keys()
            .map(|e| e.as_string())
            .collect::<Vec<String>>();
        Ok(Exp::List(keys.into_iter().map(Exp::String).collect()))
    }

    pub fn handle_len(&self, _args: &[Exp]) -> Result<Exp> {
        Ok(Exp::Number(self.inner.read().len() as f64))
    }
}

impl NativeObject for HashMapObject {
    fn get_type_name(&self) -> &'static str {
        "HashMap"
    }

    fn call_method(&self, method_name: &str, args: &[Exp]) -> Result<Exp> {
        match method_name {
            "insert" => self.handle_insert(args),
            "get" => self.handle_get(args),
            "remove" => self.handle_remove(args),
            "keys" => self.handle_keys(args),
            "len" => self.handle_len(args),
            _ => Err(format!("HashMap has no method {}", method_name).into()),
        }
    }
}
