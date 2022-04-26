//! This module contains types for all kinds of errors that can occur when
//! converting between http requests / responses and ruma's representation of
//! matrix API requests / responses.

use std::{error::Error as StdError, fmt};

use bytes::BufMut;
use serde_json::{from_slice as from_json_slice, Value as JsonValue};
use thiserror::Error;

use super::{EndpointError, MatrixVersion, OutgoingResponse};

/// A general-purpose Matrix error type consisting of an HTTP status code and a JSON body.
///
/// Note that individual `ruma-*-api` crates may provide more specific error types.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Debug)]
pub struct MatrixError {
    /// The http response's status code.
    pub status_code: http::StatusCode,

    /// The http response's body.
    pub body: JsonValue,
}

impl fmt::Display for MatrixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] ", self.status_code.as_u16())?;
        fmt::Display::fmt(&self.body, f)
    }
}

impl StdError for MatrixError {}

impl OutgoingResponse for MatrixError {
    fn try_into_http_response<T: Default + BufMut>(
        self,
    ) -> Result<http::Response<T>, IntoHttpError> {
        http::Response::builder()
            .header(http::header::CONTENT_TYPE, "application/json")
            .status(self.status_code)
            .body(crate::serde::json_to_buf(&self.body)?)
            .map_err(Into::into)
    }
}

impl EndpointError for MatrixError {
    fn try_from_http_response<T: AsRef<[u8]>>(
        response: http::Response<T>,
    ) -> Result<Self, DeserializationError> {
        Ok(Self {
            status_code: response.status(),
            body: from_json_slice(response.body().as_ref())?,
        })
    }
}

/// An error when converting one of ruma's endpoint-specific request or response
/// types to the corresponding http type.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum IntoHttpError {
    /// Tried to create an authentication request without an access token.
    #[error("no access token given, but this endpoint requires one")]
    NeedsAuthentication,

    /// Tried to create a request with an old enough version, for which no unstable endpoint
    /// exists.
    ///
    /// This is also a fallback error for if the version is too new for this endpoint.
    #[error(
        "endpoint was not supported by server-reported versions, \
         but no unstable path to fall back to was defined"
    )]
    NoUnstablePath,

    /// Tried to create a request with [`MatrixVersion`]s for all of which this endpoint was
    /// removed.
    #[error("could not create any path variant for endpoint, as it was removed in version {0}")]
    EndpointRemoved(MatrixVersion),

    /// JSON serialization failed.
    #[error("JSON serialization failed: {0}")]
    Json(#[from] serde_json::Error),

    /// Query parameter serialization failed.
    #[error("query parameter serialization failed: {0}")]
    Query(#[from] crate::serde::urlencoded::ser::Error),

    /// Header serialization failed.
    #[error("header serialization failed: {0}")]
    Header(#[from] http::header::InvalidHeaderValue),

    /// HTTP request construction failed.
    #[error("HTTP request construction failed: {0}")]
    Http(#[from] http::Error),
}

/// An error when converting a http request to one of ruma's endpoint-specific request types.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum FromHttpRequestError {
    /// Deserialization failed
    #[error("deserialization failed: {0}")]
    Deserialization(DeserializationError),

    /// HTTP method mismatch
    #[error("http method mismatch: expected {expected}, received: {received}")]
    MethodMismatch {
        /// expected http method
        expected: http::method::Method,
        /// received http method
        received: http::method::Method,
    },
}

impl<T> From<T> for FromHttpRequestError
where
    T: Into<DeserializationError>,
{
    fn from(err: T) -> Self {
        Self::Deserialization(err.into())
    }
}

/// An error when converting a http response to one of Ruma's endpoint-specific response types.
#[derive(Debug)]
#[non_exhaustive]
pub enum FromHttpResponseError<E> {
    /// Deserialization failed
    Deserialization(DeserializationError),

    /// The server returned a non-success status
    Server(ServerError<E>),
}

impl<E> FromHttpResponseError<E> {
    /// Map `FromHttpResponseError<E>` to `FromHttpResponseError<F>` by applying a function to a
    /// contained `Server` value, leaving a `Deserialization` value untouched.
    pub fn map<F>(
        self,
        f: impl FnOnce(ServerError<E>) -> ServerError<F>,
    ) -> FromHttpResponseError<F> {
        match self {
            Self::Deserialization(d) => FromHttpResponseError::Deserialization(d),
            Self::Server(s) => FromHttpResponseError::Server(f(s)),
        }
    }
}

impl<E, F> FromHttpResponseError<Result<E, F>> {
    /// Transpose `FromHttpResponseError<Result<E, F>>` to `Result<FromHttpResponseError<E>, F>`.
    pub fn transpose(self) -> Result<FromHttpResponseError<E>, F> {
        match self {
            Self::Deserialization(d) => Ok(FromHttpResponseError::Deserialization(d)),
            Self::Server(s) => s.transpose().map(FromHttpResponseError::Server),
        }
    }
}

impl<E: fmt::Display> fmt::Display for FromHttpResponseError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Deserialization(err) => write!(f, "deserialization failed: {}", err),
            Self::Server(err) => write!(f, "the server returned an error: {}", err),
        }
    }
}

impl<E> From<ServerError<E>> for FromHttpResponseError<E> {
    fn from(err: ServerError<E>) -> Self {
        Self::Server(err)
    }
}

impl<E, T> From<T> for FromHttpResponseError<E>
where
    T: Into<DeserializationError>,
{
    fn from(err: T) -> Self {
        Self::Deserialization(err.into())
    }
}

impl<E: StdError> StdError for FromHttpResponseError<E> {}

/// An error was reported by the server (HTTP status code 4xx or 5xx)
#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum ServerError<E> {
    /// An error that is expected to happen under certain circumstances and
    /// that has a well-defined structure
    Known(E),

    /// An error of unexpected type of structure
    Unknown(DeserializationError),
}

impl<E> ServerError<E> {
    /// Map `ServerError<E>` to `ServerError<F>` by applying a function to a contained `Known`
    /// value, leaving an `Unknown` value untouched.
    pub fn map<F>(self, f: impl FnOnce(E) -> F) -> ServerError<F> {
        match self {
            Self::Known(k) => ServerError::Known(f(k)),
            Self::Unknown(u) => ServerError::Unknown(u),
        }
    }
}

impl<E, F> ServerError<Result<E, F>> {
    /// Transpose `ServerError<Result<E, F>>` to `Result<ServerError<E>, F>`.
    pub fn transpose(self) -> Result<ServerError<E>, F> {
        match self {
            Self::Known(Ok(k)) => Ok(ServerError::Known(k)),
            Self::Known(Err(e)) => Err(e),
            Self::Unknown(u) => Ok(ServerError::Unknown(u)),
        }
    }
}

impl<E: fmt::Display> fmt::Display for ServerError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::Known(e) => fmt::Display::fmt(e, f),
            ServerError::Unknown(res_err) => fmt::Display::fmt(res_err, f),
        }
    }
}

impl<E: StdError> StdError for ServerError<E> {}

/// An error when converting a http request / response to one of ruma's endpoint-specific request /
/// response types.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DeserializationError {
    /// Encountered invalid UTF-8.
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),

    /// JSON deserialization failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// Query parameter deserialization failed.
    #[error(transparent)]
    Query(#[from] crate::serde::urlencoded::de::Error),

    /// Got an invalid identifier.
    #[error(transparent)]
    Ident(#[from] crate::IdParseError),

    /// Header value deserialization failed.
    #[error(transparent)]
    Header(#[from] HeaderDeserializationError),
}

impl From<std::convert::Infallible> for DeserializationError {
    fn from(err: std::convert::Infallible) -> Self {
        match err {}
    }
}

impl From<http::header::ToStrError> for DeserializationError {
    fn from(err: http::header::ToStrError) -> Self {
        Self::Header(HeaderDeserializationError::ToStrError(err))
    }
}

/// An error with the http headers.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum HeaderDeserializationError {
    /// Failed to convert `http::header::HeaderValue` to `str`.
    #[error("{0}")]
    ToStrError(http::header::ToStrError),

    /// The given required header is missing.
    #[error("missing header `{0}`")]
    MissingHeader(String),
}

/// An error that happens when Ruma cannot understand a Matrix version.
#[derive(Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct UnknownVersionError;

impl fmt::Display for UnknownVersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "version string was unknown")
    }
}

impl StdError for UnknownVersionError {}
