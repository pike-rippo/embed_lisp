use std::{
    fmt::Display,
    hash::{Hash, Hasher},
    ptr,
};

#[cfg(feature = "async")]
use crate::exp::{FutureExp, TaskExp};
use crate::{
    Error,
    environment::SharedEnv,
    error::Result,
    evaluator::Evaluator,
    exp::{LambdaExp, namespace::NameSpace},
    flow::{EvalFlow, EvalResult},
    typedef::Shared,
};

pub trait Callable: Display {
    fn call_method(&self, name: &str, args: &[Exp]) -> EvalResult;
}

#[cfg(not(feature = "async"))]
pub type SharedNativeObject = Shared<dyn Callable>;
#[cfg(feature = "async")]
pub type SharedNativeObject = Shared<dyn Callable + Send + Sync>;

pub type PrimitiveFunction = fn(&[Exp], &SharedEnv, &Evaluator) -> EvalResult;

#[derive(Clone)]
pub enum Exp {
    Nil,
    Number(f64),
    Bool(bool),
    String(String),
    List(Vec<Exp>),
    DottedList(Vec<Exp>, Shared<Exp>),
    Symbol(String),
    Primitive(PrimitiveFunction),
    Lambda(LambdaExp),
    RecurLambda(Shared<Exp>, LambdaExp),
    Macro(LambdaExp),
    Native(SharedNativeObject),
    Namespace(Shared<NameSpace>),
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
            (List(a), List(b)) => a == b,
            (DottedList(a1, a2), DottedList(b1, b2)) => a1 == b1 && a2 == b2,
            (Symbol(a), Symbol(b)) => a == b,
            (Primitive(a), Primitive(b)) => std::ptr::fn_addr_eq(*a, *b),
            (Lambda(a), Lambda(b)) => a == b,
            (Macro(a), Macro(b)) => a == b,
            (Native(a), Native(b)) => ptr::eq(&**a, &**b),
            (Namespace(a), Namespace(b)) => ptr::eq(&**a, &**b),
            #[cfg(feature = "async")]
            (Future(a), Future(b)) => a == b,
            #[cfg(feature = "async")]
            (Task(a), Task(b)) => a == b,

            _ => false,
        }
    }
}

impl Eq for Exp {}

impl Hash for Exp {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use Exp::*;
        match self {
            Nil => 0.hash(state),
            Number(n) => n.to_bits().hash(state),
            Bool(b) => b.hash(state),
            String(s) => s.hash(state),
            Symbol(s) => s.hash(state),
            List(l) => l.hash(state),
            _ => {}
        }
    }
}

impl std::fmt::Debug for Exp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "Exp::Nil"),
            Self::Number(n) => write!(f, "Exp::Number({})", n),
            Self::Bool(b) => write!(f, "Exp::Bool({})", b),
            Self::String(s) => write!(f, "Exp::String(\"{}\")", s),
            Self::List(list) => write!(f, "Exp::List({:?})", list),
            Self::DottedList(args, tail) => write!(
                f,
                "({} ... {:?})",
                args.iter()
                    .map(|e| format!("{:?}", e))
                    .collect::<Vec<String>>()
                    .join(" "),
                tail
            ),
            Self::Symbol(s) => write!(f, "Exp::Symbol({})", s),
            Self::Primitive(function) => write!(f, "Exp::Primitive({:?})", function),
            Self::Lambda(lambda) => write!(f, "Exp::Lambda({:?})", lambda),
            Self::RecurLambda(init, lambda) => {
                write!(f, "Exp::RecurLambda({:?} {:?})", init, lambda)
            }
            Self::Macro(lambda) => write!(f, "Exp::Macro({:?})", lambda),
            Self::Native(native) => write!(f, "Exp::Native({})", native),
            Self::Namespace(_) => write!(f, "Exp::Namespace()"),
            #[cfg(feature = "async")]
            Self::Future(future) => write!(f, "Exp::Future{{ {:?} }}", future),
            #[cfg(feature = "async")]
            Self::Task(task) => write!(f, "Task {{ {} }}", task.status()),
        }
    }
}

impl std::fmt::Display for Exp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Nil => "Nil".to_string(),
            Self::Number(n) => n.to_string(),
            Self::Bool(b) => b.to_string(),
            Exp::String(s) => s.clone(),
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
            Self::DottedList(args, tail) => format!(
                "({} ... {})",
                args.iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<String>>()
                    .join(" "),
                tail
            ),
            Self::Symbol(s) => s.clone(),
            Self::Primitive(_) => format!("Primitive"),
            Self::Lambda(lambda) => {
                format!(
                    "Lambda {{ {} -> {} }}",
                    lambda.params_exp,
                    lambda
                        .body_exp
                        .iter()
                        .map(|e| format!("{}", e))
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            }
            Self::RecurLambda(init, lambda) => {
                format!(
                    "Lambda {{ {} {} -> {} }}",
                    init,
                    lambda.params_exp,
                    lambda
                        .body_exp
                        .iter()
                        .map(|e| format!("{}", e))
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            }
            Self::Macro(lambda) => {
                format!(
                    "Macro {{ {} -> {} }}",
                    lambda.params_exp,
                    lambda
                        .body_exp
                        .iter()
                        .map(|e| format!("{}", e))
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            }
            Self::Native(native) => format!("Native {{ {} }}", native),
            Self::Namespace(_) => "Namespace".to_string(),
            #[cfg(feature = "async")]
            Self::Future(future) => format!("Future {{ {} }}", future),
            #[cfg(feature = "async")]
            Self::Task(task) => format!("Task {{ {} }}", task.status()),
        };
        write!(f, "{}", str)
    }
}

impl Exp {
    pub fn quote() -> Self {
        Exp::Symbol("quote".to_string())
    }

    pub fn value_flow(self) -> EvalFlow {
        EvalFlow::Value(self)
    }

    pub fn return_flow(self) -> EvalFlow {
        EvalFlow::Return(self)
    }

    pub fn wrap_quote(&self) -> Self {
        Exp::List(vec![Exp::quote(), self.clone()])
    }

    pub fn is_truthy(&self) -> bool {
        // !matches!(self, Exp::Bool(false) | Exp::Nil)
        match self {
            Exp::Bool(false) | Exp::Nil => false,
            Exp::List(list) if list.is_empty() => false,
            _ => true,
        }
    }

    pub fn is_dotted_list(&self) -> bool {
        matches!(self, Self::DottedList(_, _))
    }

    pub fn is_namespace(&self) -> bool {
        matches!(self, Self::Namespace(_))
    }

    pub fn is_quote(&self) -> bool {
        matches!(self, Self::Symbol(s) if &s[..] == "quote")
    }

    pub fn is_quoted(&self) -> bool {
        match self {
            Exp::List(list) => {
                matches!(
                    (list.first(), list.get(1)), (Some(Exp::Symbol(s)), _) if list.len() == 2 && s == "quote")
            }
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
            Exp::DottedList(_, _) => "DottedList".to_string(),
            Exp::Symbol(_) => "Symbol".to_string(),
            Exp::Primitive(_) => "Primitive".to_string(),
            Exp::Lambda(_) => "Lambda".to_string(),
            Exp::RecurLambda(_, _) => "RecurLambda".to_string(),
            Exp::Macro(_) => "Macro".to_string(),
            Exp::Native(native) => format!("Native {{ {} }}", native),
            Exp::Namespace(_) => format!("Namespace"),
            #[cfg(feature = "async")]
            Exp::Future(_) => "Future".to_string(),
            #[cfg(feature = "async")]
            Exp::Task(_) => "Task".to_string(),
        }
    }

    pub fn as_string_exp(&self) -> Exp {
        Exp::String(format!("{}", self))
    }

    pub fn as_string(&self) -> String {
        format!("{}", self)
    }

    pub fn check_key_allowed(&self) -> Result<()> {
        #[cfg(not(feature = "async"))]
        match self {
            Exp::Native(_) | Exp::Primitive(_) | Exp::Lambda(_) | Exp::Macro(_) => {
                Err(Error::reason("invalid key type"))
            }
            _ => Ok(()),
        }

        #[cfg(feature = "async")]
        match self {
            Exp::Native(_)
            | Exp::Primitive(_)
            | Exp::Lambda(_)
            | Exp::Macro(_)
            | Exp::Future(_)
            | Exp::Task(_) => Err(Error::reason("invalid key type")),
            _ => Ok(()),
        }
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
