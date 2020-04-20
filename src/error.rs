use std::{borrow::Cow, error, fmt, str};

use serde::ser;

pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned during serializing to `application/x-www-form-urlencoded`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Custom(Cow<'static, str>),
    Utf8(str::Utf8Error),
}

impl Error {
    pub fn done() -> Self {
        Error::Custom("this pair has already been serialized".into())
    }

    pub fn not_done() -> Self {
        Error::Custom("this pair has not yet been serialized".into())
    }

    pub fn unsupported_pair() -> Self {
        Error::Custom("unsupported pair".into())
    }

    pub fn top_level() -> Self {
        let msg = "top-level serializer supports only maps and structs";
        Error::Custom(msg.into())
    }

    pub fn no_key() -> Self {
        let msg = "tried to serialize a value before serializing key";
        Error::Custom(msg.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Custom(ref msg) => msg.fmt(f),
            Error::Utf8(ref err) => write!(f, "invalid UTF-8: {}", err),
        }
    }
}

impl error::Error for Error {
    /// The lower-level cause of this error, in the case of a `Utf8` error.
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::Custom(_) => None,
            Error::Utf8(ref err) => Some(err),
        }
    }

    /// The lower-level source of this error, in the case of a `Utf8` error.
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::Custom(_) => None,
            Error::Utf8(ref err) => Some(err),
        }
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Custom(format!("{}", msg).into())
    }
}
