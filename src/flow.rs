use std::fmt::Display;

use crate::{Exp, err, error::Result};

pub type EvalResult = Result<EvalFlow>;

pub enum EvalFlow {
    Value(Exp),
    Return(Exp),
    Break,
    Continue,
}

impl EvalFlow {
    pub fn try_unwrap(&self) -> Result<Exp> {
        match self {
            EvalFlow::Value(exp) | EvalFlow::Return(exp) => Ok(exp.clone()),
            _ => err!("unexpected control flow in unwrap"),
        }
    }
}

impl Display for EvalFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalFlow::Break => write!(f, "break"),
            EvalFlow::Continue => write!(f, "continue"),
            EvalFlow::Value(v) => write!(f, "{}", v),
            EvalFlow::Return(v) => write!(f, "{}", v),
        }
    }
}
