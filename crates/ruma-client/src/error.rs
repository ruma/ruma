//! Error conditions.

use std::fmt::{self, Debug, Display, Formatter};

use ruma_common::api::error::{FromHttpResponseError, IntoHttpError};

/// An error that can occur during client operations.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error<E, F> {
    /// Queried endpoint requires authentication but was called on an anonymous client.
    AuthenticationRequired,

    /// Construction of the HTTP request failed (this should never happen).
    IntoHttp(IntoHttpError),

    /// The request's URL is invalid (this should never happen).
    Url(http::Error),

    /// Couldn't obtain an HTTP response (e.g. due to network or DNS issues).
    Response(E),

    /// Converting the HTTP response to one of ruma's types failed.
    FromHttpResponse(FromHttpResponseError<F>),
}

#[cfg(feature = "client-api")]
impl<E> Error<E, ruma_client_api::Error> {
    /// If `self` is a server error in the `errcode` + `error` format expected
    /// for client-server API endpoints, returns the error kind (`errcode`).
    pub fn error_kind(&self) -> Option<&ruma_client_api::error::ErrorKind> {
        use as_variant::as_variant;
        use ruma_client_api::error::FromHttpResponseErrorExt as _;

        as_variant!(self, Self::FromHttpResponse)?.error_kind()
    }
}

impl<E: Display, F: Display> Display for Error<E, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthenticationRequired => {
                write!(f, "The queried endpoint requires authentication but was called with an anonymous client.")
            }
            Self::IntoHttp(err) => write!(f, "HTTP request construction failed: {err}"),
            Self::Url(err) => write!(f, "Invalid URL: {err}"),
            Self::Response(err) => write!(f, "Couldn't obtain a response: {err}"),
            Self::FromHttpResponse(err) => write!(f, "HTTP response conversion failed: {err}"),
        }
    }
}

impl<E, F> From<IntoHttpError> for Error<E, F> {
    fn from(err: IntoHttpError) -> Self {
        Error::IntoHttp(err)
    }
}

#[doc(hidden)]
impl<E, F> From<http::uri::InvalidUri> for Error<E, F> {
    fn from(err: http::uri::InvalidUri) -> Self {
        Error::Url(err.into())
    }
}

#[doc(hidden)]
impl<E, F> From<http::uri::InvalidUriParts> for Error<E, F> {
    fn from(err: http::uri::InvalidUriParts) -> Self {
        Error::Url(err.into())
    }
}

impl<E, F> From<FromHttpResponseError<F>> for Error<E, F> {
    fn from(err: FromHttpResponseError<F>) -> Self {
        Error::FromHttpResponse(err)
    }
}

impl<E: Debug + Display, F: Debug + Display> std::error::Error for Error<E, F> {}
