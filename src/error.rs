//! Error conditions.

use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};

use ruma_api::error::{FromHttpResponseError, IntoHttpError};

use crate::api;

/// An error that can occur during client operations.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Queried endpoint requires authentication but was called on an anonymous client.
    AuthenticationRequired,
    /// Construction of the HTTP request failed (this should never happen).
    IntoHttp(IntoHttpError),
    /// The request's URL is invalid (this should never happen).
    Url(UrlError),
    /// Couldn't obtain an HTTP response (e.g. due to network or DNS issues).
    Response(ResponseError),
    /// Converting the HTTP response to one of ruma's types failed.
    FromHttpResponse(FromHttpResponseError<api::Error>),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::AuthenticationRequired => {
                write!(f, "The queried endpoint requires authentication but was called with an anonymous client.")
            }
            Self::IntoHttp(err) => write!(f, "HTTP request construction failed: {}", err),
            Self::Url(UrlError(err)) => write!(f, "Invalid URL: {}", err),
            Self::Response(ResponseError(err)) => write!(f, "Couldn't obtain a response: {}", err),
            // FIXME: ruma-client-api's Error type currently doesn't implement
            //        `Display`, update this when it does.
            Self::FromHttpResponse(_) => write!(f, "HTTP response conversion failed"),
        }
    }
}

impl From<IntoHttpError> for Error {
    fn from(err: IntoHttpError) -> Self {
        Error::IntoHttp(err)
    }
}

#[doc(hidden)]
impl From<http::uri::InvalidUri> for Error {
    fn from(err: http::uri::InvalidUri) -> Self {
        Error::Url(UrlError(err))
    }
}

#[doc(hidden)]
impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Self {
        Error::Response(ResponseError(err))
    }
}

impl From<FromHttpResponseError<api::Error>> for Error {
    fn from(err: FromHttpResponseError<api::Error>) -> Self {
        Error::FromHttpResponse(err)
    }
}

impl StdError for Error {}

#[derive(Debug)]
pub struct UrlError(http::uri::InvalidUri);

#[derive(Debug)]
pub struct ResponseError(hyper::Error);
