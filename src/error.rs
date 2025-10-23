use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[macro_export]
macro_rules! ok {
    ($value:expr) => {
        return std::result::Result::Ok($value.into())
    };
}

#[macro_export]
macro_rules! err {
    ($value:expr) => {
        return std::result::Result::Err($value.into())
    };
}

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("error: {0}")]
    Reason(String),
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Reason(value)
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Reason(value.to_string())
    }
}
