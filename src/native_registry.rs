use std::collections::HashMap;

use parking_lot::RwLock;

use crate::{Exp, error::SyntaxError, flow::EvalResult, typedef::Shared};

pub type NativeCreator = fn(&[Exp]) -> EvalResult;

#[derive(Clone)]
pub struct NativeRegistry {
    inner: Shared<RwLock<HashMap<String, NativeCreator>>>,
}

impl NativeRegistry {
    pub fn new() -> Self {
        Self {
            inner: Shared::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register(&self, name: &str, creator: NativeCreator) {
        self.inner.write().insert(name.to_string(), creator);
    }

    pub fn create(&self, name: &str, args: &[Exp]) -> EvalResult {
        let guard = self.inner.read();
        let creator = guard
            .get(name)
            .ok_or_else(|| SyntaxError::Reason(format!("unknown native type: '{}'", name)))?;
        creator(args)
    }

    pub fn keys(&self) -> Vec<String> {
        self.inner.read().keys().cloned().collect()
    }
}
