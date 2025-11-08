use std::fmt::Display;

use crate::{Error, Exp, error::Result};

pub type EvalResult = Result<EvalFlow>;

pub enum EvalFlow {
    Value(Exp),
    TailCall(Vec<Exp>),
    Return(Exp),
    Break,
    Continue,
}

impl EvalFlow {
    pub fn try_unwrap(&self) -> Result<Exp> {
        match self {
            EvalFlow::Value(exp) | EvalFlow::Return(exp) => Ok(exp.clone()),
            _ => return Err(Error::reason("unexpected control flow in unwrap")),
        }
    }
}

impl Display for EvalFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalFlow::Value(v) => write!(f, "{}", v),
            EvalFlow::TailCall(_) => write!(f, "TailCall"),
            EvalFlow::Return(v) => write!(f, "{}", v),
            EvalFlow::Continue => write!(f, "continue"),
            EvalFlow::Break => write!(f, "break"),
        }
    }
}
