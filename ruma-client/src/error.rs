//! Error conditions.

use std::fmt::{self, Debug, Display, Formatter};

use ruma_api::error::{FromHttpResponseError, IntoHttpError};

/// An error that can occur during client operations.
#[derive(Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum Error<E> {
    /// Queried endpoint requires authentication but was called on an anonymous client.
    AuthenticationRequired,

    /// Construction of the HTTP request failed (this should never happen).
    IntoHttp(IntoHttpError),

    /// The request's URL is invalid (this should never happen).
    Url(UrlError),

    /// Couldn't obtain an HTTP response (e.g. due to network or DNS issues).
    Response(ResponseError),

    /// Converting the HTTP response to one of ruma's types failed.
    FromHttpResponse(FromHttpResponseError<E>),
}

impl<E: Display> Display for Error<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthenticationRequired => {
                write!(f, "The queried endpoint requires authentication but was called with an anonymous client.")
            }
            Self::IntoHttp(err) => write!(f, "HTTP request construction failed: {}", err),
            Self::Url(UrlError(err)) => write!(f, "Invalid URL: {}", err),
            Self::Response(ResponseError(err)) => write!(f, "Couldn't obtain a response: {}", err),
            Self::FromHttpResponse(err) => write!(f, "HTTP response conversion failed: {}", err),
        }
    }
}

impl<E> From<IntoHttpError> for Error<E> {
    fn from(err: IntoHttpError) -> Self {
        Error::IntoHttp(err)
    }
}

#[doc(hidden)]
impl<E> From<http::uri::InvalidUri> for Error<E> {
    fn from(err: http::uri::InvalidUri) -> Self {
        Error::Url(UrlError(err.into()))
    }
}

#[doc(hidden)]
impl<E> From<http::uri::InvalidUriParts> for Error<E> {
    fn from(err: http::uri::InvalidUriParts) -> Self {
        Error::Url(UrlError(err.into()))
    }
}

#[doc(hidden)]
impl<E> From<hyper::Error> for Error<E> {
    fn from(err: hyper::Error) -> Self {
        Error::Response(ResponseError(err))
    }
}

impl<E> From<FromHttpResponseError<E>> for Error<E> {
    fn from(err: FromHttpResponseError<E>) -> Self {
        Error::FromHttpResponse(err)
    }
}

impl<E: Debug + Display> std::error::Error for Error<E> {}

#[derive(Debug)]
pub struct UrlError(http::Error);

#[derive(Debug)]
pub struct ResponseError(hyper::Error);
