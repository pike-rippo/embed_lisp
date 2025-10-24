use std::fmt::format;

#[cfg(feature = "async")]
use tokio::{sync::Mutex, task::JoinHandle};

use crate::{
    error::Result,
    evaluator::Evaluator,
    lambda::LambdaExp,
    native::{self, NativeObject},
    typedef::{Shared, SharedEnv},
};
#[cfg(feature = "async")]
use crate::{future::FutureExp, task::TaskExp};

pub type BuiltinFunction = fn(&[Exp], &SharedEnv, &Evaluator) -> Result<Exp>;

#[derive(Clone)]
pub enum Exp {
    Nil,
    Number(f64),
    Bool(bool),
    String(String),
    List(Vec<Exp>),
    Symbol(String),
    Function(BuiltinFunction),
    Lambda(LambdaExp),
    Macro(LambdaExp),
    #[cfg(not(feature = "async"))]
    Native(Shared<dyn NativeObject>),
    #[cfg(feature = "async")]
    Native(Shared<dyn NativeObject + Send + Sync>),
    #[cfg(feature = "async")]
    Future(FutureExp),
    #[cfg(feature = "async")]
    Task(TaskExp),
}

impl PartialEq for Exp {
    fn eq(&self, other: &Self) -> bool {
        use Exp::*;
        match (self, other) {
            (Nil, Nil) => true,
            (Number(a), Number(b)) => a == b,
            (Bool(a), Bool(b)) => a == b,
            (String(a), String(b)) => a == b,
            (Symbol(a), Symbol(b)) => a == b,
            (List(a), List(b)) => a == b,
            (Native(_), Native(_)) => false,
            _ => false,
        }
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl std::fmt::Debug for Exp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Exp::Native(obj) => write!(f, "<native:{}>", obj.get_type_name()),
            other => write!(f, "{:?}", other),
        }
    }
}

impl std::fmt::Display for Exp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Nil => "Nil".to_string(),
            Self::Number(n) => n.to_string(),
            Self::Bool(b) => b.to_string(),
            Exp::String(s) => format!("\"{}\"", s),
            Self::List(list) => {
                if let Some(Exp::Symbol(s)) = list.first() {
                    let xs: Vec<_> = list[1..].iter().map(|x| x.to_string()).collect();
                    match &s[..] {
                        "quote" => format!("'{}", xs.join(" ")),
                        "unquote" => format!(",{}", xs.join(" ")),
                        "quasiquote" => format!("`{}", xs.join(" ")),
                        "unquote-splicing" => format!(",@{}", xs.join(" ")),
                        _ => {
                            if xs.is_empty() {
                                format!("({})", s)
                            } else {
                                format!("({} {})", s, xs.join(" "))
                            }
                        }
                    }
                } else {
                    let xs: Vec<_> = list.iter().map(|x| x.to_string()).collect();
                    format!("({})", xs.join(" "))
                }
            }
            Self::Symbol(s) => s.clone(),
            Self::Function(_) => "Function".to_string(),
            Self::Lambda(lambda) => {
                format!("Lambda {{ {} -> {} }}", lambda.params_exp, lambda.body_exp)
            }
            Self::Macro(lambda) => {
                format!("Macro {{ {} -> {} }}", lambda.params_exp, lambda.body_exp)
            }
            Self::Native(native) => format!("Native {{ {} }}", native.get_type_name()),
            #[cfg(feature = "async")]
            Self::Future(_) => "Future {}".to_string(),
            #[cfg(feature = "async")]
            Self::Task(future) => if future.is_ready() {
                "Task { Ready }"
            } else {
                "Task { Pending }"
            }
            .to_string(),
        };
        write!(f, "{}", str)
    }
}

impl Exp {
    pub fn quote() -> Self {
        Exp::Symbol("quote".to_string())
    }

    pub fn wrap_quote(&self) -> Self {
        Exp::List(vec![Exp::quote(), self.clone()])
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Exp::Bool(false) => false,
            Exp::Nil => false,
            _ => true,
        }
    }

    pub fn is_quote(&self) -> bool {
        match self {
            Self::Symbol(s) if &s[..] == "quote" => true,
            _ => false,
        }
    }

    pub fn is_quoted(&self) -> bool {
        match self {
            Exp::List(list) => match (list.first(), list.get(1)) {
                (Some(Exp::Symbol(s)), _) if list.len() == 2 && s == "quote" => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn unwrap_quote(&self) -> Self {
        match self {
            Self::List(list) => {
                if list.len() == 2 && list[0].is_quote() {
                    list[1].clone()
                } else {
                    self.clone()
                }
            }
            _ => self.clone(),
        }
    }

    pub fn type_of(&self) -> String {
        match self {
            Exp::Nil => "Nil".to_string(),
            Exp::Number(_) => "Number".to_string(),
            Exp::Bool(_) => "Bool".to_string(),
            Exp::String(_) => "String".to_string(),
            Exp::List(_) => "List".to_string(),
            Exp::Symbol(_) => "Symbol".to_string(),
            Exp::Function(_) => "Function".to_string(),
            Exp::Lambda(_) => "Lambda".to_string(),
            Exp::Macro(_) => "Macro".to_string(),
            Exp::Native(native) => format!("Native {{ {} }}", native.get_type_name()),
            #[cfg(feature = "async")]
            Exp::Future(_) => "Future".to_string(),
            #[cfg(feature = "async")]
            Exp::Task(_) => "Task".to_string(),
        }
    }

    pub fn as_string_exp(&self) -> Exp {
        Exp::String(format!("{}", self))
    }
}

impl From<bool> for Exp {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<Vec<Exp>> for Exp {
    fn from(value: Vec<Exp>) -> Self {
        Self::List(value)
    }
}

impl From<f64> for Exp {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}
