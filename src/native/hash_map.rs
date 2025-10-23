use std::{collections::HashMap, sync::RwLock};

use crate::{error::Result, expression::Exp, native::NativeObject, ok, read_error, write_error};

pub struct HashMapObject {
    inner: RwLock<HashMap<String, Exp>>,
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

        let (k, v) = match (&args[0], &args[1]) {
            (Exp::String(k), v) => (k.clone(), v.clone()),
            _ => return Err("insert expected string key and value".into()),
        };

        self.inner.write().or(write_error!())?.insert(k, v);

        ok!(true)
    }

    pub fn handle_get(&self, args: &[Exp]) -> Result<Exp> {
        if args.len() != 1 {
            return Err("get expected 1 argument".into());
        }

        if let Exp::String(k) = &args[0] {
            Ok(self
                .inner
                .read()
                .or(read_error!())?
                .get(k)
                .cloned()
                .unwrap_or(Exp::Nil))
        } else {
            Err("get expected string key".into())
        }
    }

    pub fn handle_keys(&self, _args: &[Exp]) -> Result<Exp> {
        let keys = self
            .inner
            .read()
            .or(read_error!())?
            .keys()
            .cloned()
            .collect::<Vec<String>>();
        Ok(Exp::List(keys.into_iter().map(Exp::String).collect()))
    }

    pub fn handle_len(&self, _args: &[Exp]) -> Result<Exp> {
        Ok(Exp::Number(
            self.inner.read().or(read_error!())?.len() as f64
        ))
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
            "keys" => self.handle_keys(args),
            "len" => self.handle_len(args),
            _ => Err(format!("HashMap has no method {}", method_name).into()),
        }
    }
}
