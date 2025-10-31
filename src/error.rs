use thiserror::Error;

use crate::flow::EvalFlow;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("error: {0}")]
    Reason(String),

    #[error("parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("syntax error: {0}")]
    Syntax(#[from] SyntaxError),
}

impl Error {
    pub fn reason(reason: &str) -> Error {
        Error::Reason(reason.to_string())
    }

    pub fn into_err(self) -> std::result::Result<EvalFlow, Error> {
        Err(self)
    }
}

#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("{0}")]
    Reason(String),

    #[error("unterminated string literal")]
    UnterminatedStringLiteral,

    #[error("unexpected '{0}'")]
    UnexpectedToken(String),
}

impl ParseError {
    pub fn reason(reason: &str) -> Error {
        ParseError::Reason(reason.to_string()).into()
    }

    pub fn unterminated_string_literal() -> Error {
        ParseError::UnterminatedStringLiteral.into()
    }

    pub fn unexpected_token(token: &str) -> Error {
        ParseError::UnexpectedToken(token.to_string()).into()
    }
}

#[derive(Error, Debug, Clone)]
pub enum SyntaxError {
    #[error("{0}")]
    Reason(String),

    #[error("'{0}' expects at least {1} arguments, but received {2}")]
    NotEnoughArgs(String, usize, usize),

    #[error("'{0}' expects a maximum of {1} arguments, but received {2}")]
    TooManyArgs(String, usize, usize),

    #[error("'{0}' expects {1} arguments, but received {2}")]
    InvalidArgsSize(String, usize, usize),

    #[error("'{0}' expects only {1}")]
    InvalidArgsType(String, String),

    #[error("'{0}' expects {1} as {2} argument")]
    InvalidArgsTypeNth(String, String, String),

    #[error("'{0}' is not defined")]
    UnboundSymbol(String),

    #[error("unexpected form: {0}")]
    UnexpectedForm(String),

    #[error("'{0}' has not such method: '{1}'")]
    NoSuchMethod(String, String),
}

impl SyntaxError {
    pub fn reason(reason: &str) -> Error {
        SyntaxError::Reason(reason.to_string()).into()
    }

    pub fn not_enough_args(name: &str, expected: usize, received: usize) -> Error {
        SyntaxError::NotEnoughArgs(name.to_string(), expected, received).into()
    }

    pub fn too_many_args(name: &str, expected: usize, received: usize) -> Error {
        SyntaxError::TooManyArgs(name.to_string(), expected, received).into()
    }

    pub fn invalid_args_size(name: &str, expected: usize, received: usize) -> Error {
        SyntaxError::InvalidArgsSize(name.to_string(), expected, received).into()
    }

    pub fn invalid_args_type(name: &str, typ: &str) -> Error {
        SyntaxError::InvalidArgsType(name.to_string(), typ.to_string()).into()
    }

    pub fn invalid_args_type_nth(name: &str, typ: &str, nth: usize) -> Error {
        let nth = match nth {
            0 => panic!("Invalid argument index"),
            1 => "first".to_string(),
            2 => "second".to_string(),
            3 => "third".to_string(),
            _ => format!("{}th", nth),
        };
        SyntaxError::InvalidArgsTypeNth(name.to_string(), typ.to_string(), nth).into()
    }

    pub fn unbound_symbol(name: &str) -> Error {
        SyntaxError::UnboundSymbol(name.to_string()).into()
    }

    pub fn unexpected_form(name: &str) -> Error {
        SyntaxError::UnexpectedForm(name.to_string()).into()
    }

    pub fn no_such_method(name: &str, method: &str) -> Error {
        SyntaxError::NoSuchMethod(name.to_string(), method.to_string()).into()
    }
}
