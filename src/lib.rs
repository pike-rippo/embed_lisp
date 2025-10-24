#[cfg(feature = "async")]
use std::sync::LazyLock;
use std::sync::atomic::AtomicUsize;

#[cfg(feature = "async")]
use tokio::runtime::Runtime;

mod builtins;
mod environment;
mod error;
mod evaluator;
mod expression;
mod future;
mod lambda;
mod native;
mod native_registry;
mod parser;
mod replacer;
mod special_forms;
mod task;
mod typedef;

#[cfg(feature = "async")]
pub static GLOBAL_RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("failed to create tokio runtime"));

pub static GENESYM_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub use error::Error;
pub use evaluator::{Evaluator, SpecialFormFn};
pub use expression::{BuiltinFunction, Exp};
pub use native::NativeObject;

use crate::{environment::Env, error::Result, parser::Parser, typedef::SharedEnv};

pub struct Interpreter {
    parser: Parser,
    eval: Evaluator,
    env: SharedEnv,
    builtin_env: SharedEnv,
}

impl Interpreter {
    pub fn new() -> Self {
        let builtin_env = Env::builtin_env();
        Self {
            parser: Parser::new(),
            eval: Evaluator::new(),
            env: Env::new_with_builtin(builtin_env.clone()),
            builtin_env,
        }
    }

    pub fn eval(&self, exp: &Exp) -> Result<Exp> {
        self.eval.eval(exp, &self.env)
    }

    pub fn eval_str(&self, input: &str) -> Result<Exp> {
        match self.parser.parse(input) {
            Err(e) => Err(e),
            Ok(exps) => {
                let mut result = Exp::Nil;
                for exp in exps {
                    result = self.eval(&exp)?;
                }
                Ok(result)
            }
        }
    }

    pub fn register_special_form(&self, key: &str, f: SpecialFormFn) {
        self.eval.register_special_form(key, f);
    }

    pub fn register_builtin(&self, key: &str, exp: Exp) {
        self.builtin_env.define(key, exp);
    }
}
