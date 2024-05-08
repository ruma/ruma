//! This module contains types for all kinds of errors that can occur when
//! converting between http requests / responses and ruma's representation of
//! matrix API requests / responses.

use std::{error::Error as StdError, fmt, num::ParseIntError, sync::Arc};

use bytes::{BufMut, Bytes};
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
    pub body: MatrixErrorBody,
}

impl fmt::Display for MatrixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_code = self.status_code.as_u16();
        match &self.body {
            MatrixErrorBody::Json(json) => write!(f, "[{status_code}] {json}"),
            MatrixErrorBody::NotJson { .. } => write!(f, "[{status_code}] <non-json bytes>"),
        }
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
            .body(match self.body {
                MatrixErrorBody::Json(json) => crate::serde::json_to_buf(&json)?,
                MatrixErrorBody::NotJson { .. } => {
                    return Err(IntoHttpError::Json(serde::ser::Error::custom(
                        "attempted to serialize MatrixErrorBody::NotJson",
                    )));
                }
            })
            .map_err(Into::into)
    }
}

impl EndpointError for MatrixError {
    fn from_http_response<T: AsRef<[u8]>>(response: http::Response<T>) -> Self {
        let status_code = response.status();
        let body = MatrixErrorBody::from_bytes(response.body().as_ref());
        Self { status_code, body }
    }
}

/// The body of an error response.
#[derive(Clone, Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum MatrixErrorBody {
    /// A JSON body, as intended.
    Json(JsonValue),

    /// A response body that is not valid JSON.
    #[non_exhaustive]
    NotJson {
        /// The raw bytes of the response body.
        bytes: Bytes,

        /// The error from trying to deserialize the bytes as JSON.
        deserialization_error: Arc<serde_json::Error>,
    },
}

impl MatrixErrorBody {
    /// Create a `MatrixErrorBody` from the given HTTP body bytes.
    pub fn from_bytes(body_bytes: &[u8]) -> Self {
        match from_json_slice(body_bytes) {
            Ok(json) => MatrixErrorBody::Json(json),
            Err(e) => MatrixErrorBody::NotJson {
                bytes: Bytes::copy_from_slice(body_bytes),
                deserialization_error: Arc::new(e),
            },
        }
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
    Query(#[from] serde_html_form::ser::Error),

    /// Header serialization failed.
    #[error("header serialization failed: {0}")]
    Header(#[from] HeaderSerializationError),

    /// HTTP request construction failed.
    #[error("HTTP request construction failed: {0}")]
    Http(#[from] http::Error),
}

impl From<http::header::InvalidHeaderValue> for IntoHttpError {
    fn from(value: http::header::InvalidHeaderValue) -> Self {
        Self::Header(value.into())
    }
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
    Server(E),
}

impl<E> FromHttpResponseError<E> {
    /// Map `FromHttpResponseError<E>` to `FromHttpResponseError<F>` by applying a function to a
    /// contained `Server` value, leaving a `Deserialization` value untouched.
    pub fn map<F>(self, f: impl FnOnce(E) -> F) -> FromHttpResponseError<F> {
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
            Self::Server(s) => s.map(FromHttpResponseError::Server),
        }
    }
}

impl<E: fmt::Display> fmt::Display for FromHttpResponseError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Deserialization(err) => write!(f, "deserialization failed: {err}"),
            Self::Server(err) => write!(f, "the server returned an error: {err}"),
        }
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
    Query(#[from] serde_html_form::de::Error),

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

/// An error when deserializing the HTTP headers.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum HeaderDeserializationError {
    /// Failed to convert `http::header::HeaderValue` to `str`.
    #[error("{0}")]
    ToStrError(#[from] http::header::ToStrError),

    /// Failed to convert `http::header::HeaderValue` to an integer.
    #[error("{0}")]
    ParseIntError(#[from] ParseIntError),

    /// Failed to parse a HTTP date from a `http::header::Value`.
    #[error("failed to parse HTTP date")]
    InvalidHttpDate,

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

/// An error that happens when an incorrect amount of arguments have been passed to PathData parts
/// formatting.
#[derive(Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct IncorrectArgumentCount {
    /// The expected amount of arguments.
    pub expected: usize,

    /// The amount of arguments received.
    pub got: usize,
}

impl fmt::Display for IncorrectArgumentCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "incorrect path argument count, expected {}, got {}", self.expected, self.got)
    }
}

impl StdError for IncorrectArgumentCount {}

/// An error when serializing the HTTP headers.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum HeaderSerializationError {
    /// Failed to convert a header value to `http::header::HeaderValue`.
    #[error(transparent)]
    ToHeaderValue(#[from] http::header::InvalidHeaderValue),

    /// The `SystemTime` could not be converted to a HTTP date.
    ///
    /// This only happens if the `SystemTime` provided is too far in the past (before the Unix
    /// epoch) or the future (after the year 9999).
    #[error("invalid HTTP date")]
    InvalidHttpDate,
}
