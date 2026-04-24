//! This module contains types for all kinds of errors that can occur when
//! converting between http requests / responses and ruma's representation of
//! matrix API requests / responses.

use std::{error::Error as StdError, fmt, num::ParseIntError, sync::Arc};

use as_variant::as_variant;
use bytes::{BufMut, Bytes};
use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, from_slice as from_json_slice};
use thiserror::Error;

mod kind;
mod kind_serde;
#[cfg(test)]
mod tests;

pub use self::kind::*;
use super::{EndpointError, MatrixVersion, OutgoingResponse};

/// An error returned from a Matrix API endpoint.
#[derive(Clone, Debug)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct Error {
    /// The http response's status code.
    pub status_code: http::StatusCode,

    /// The http response's body.
    pub body: ErrorBody,
}

impl Error {
    /// Constructs a new `Error` with the given status code and body.
    ///
    /// This is equivalent to calling `body.into_error(status_code)`.
    pub fn new(status_code: http::StatusCode, body: ErrorBody) -> Self {
        Self { status_code, body }
    }

    /// If this is an error with a [`StandardErrorBody`], returns the [`ErrorKind`].
    pub fn error_kind(&self) -> Option<&ErrorKind> {
        as_variant!(&self.body, ErrorBody::Standard(StandardErrorBody { kind, .. }) => kind)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_code = self.status_code.as_u16();
        match &self.body {
            ErrorBody::Standard(StandardErrorBody { kind, message }) => {
                let errcode = kind.errcode();
                write!(f, "[{status_code} / {errcode}] {message}")
            }
            ErrorBody::Json(json) => write!(f, "[{status_code}] {json}"),
            ErrorBody::NotJson { .. } => write!(f, "[{status_code}] <non-json bytes>"),
        }
    }
}

impl StdError for Error {}

impl OutgoingResponse for Error {
    fn try_into_http_response<T: Default + BufMut>(
        self,
    ) -> Result<http::Response<T>, IntoHttpError> {
        let mut builder = http::Response::builder()
            .header(http::header::CONTENT_TYPE, ruma_common::http_headers::APPLICATION_JSON)
            .status(self.status_code);

        // Add data in headers.
        if let Some(ErrorKind::LimitExceeded(LimitExceededErrorData {
            retry_after: Some(retry_after),
        })) = self.error_kind()
        {
            let header_value = http::HeaderValue::try_from(retry_after)?;
            builder = builder.header(http::header::RETRY_AFTER, header_value);
        }

        builder
            .body(match self.body {
                ErrorBody::Standard(standard_body) => {
                    ruma_common::serde::json_to_buf(&standard_body)?
                }
                ErrorBody::Json(json) => ruma_common::serde::json_to_buf(&json)?,
                ErrorBody::NotJson { .. } => {
                    return Err(IntoHttpError::Json(serde::ser::Error::custom(
                        "attempted to serialize ErrorBody::NotJson",
                    )));
                }
            })
            .map_err(Into::into)
    }
}

impl EndpointError for Error {
    fn from_http_response<T: AsRef<[u8]>>(response: http::Response<T>) -> Self {
        let status = response.status();

        let body_bytes = &response.body().as_ref();
        let error_body: ErrorBody = match from_json_slice::<StandardErrorBody>(body_bytes) {
            Ok(mut standard_body) => {
                let headers = response.headers();

                if let ErrorKind::LimitExceeded(LimitExceededErrorData { retry_after }) =
                    &mut standard_body.kind
                {
                    // The Retry-After header takes precedence over the retry_after_ms field in
                    // the body.
                    if let Some(Ok(retry_after_header)) =
                        headers.get(http::header::RETRY_AFTER).map(RetryAfter::try_from)
                    {
                        *retry_after = Some(retry_after_header);
                    }
                }

                ErrorBody::Standard(standard_body)
            }
            Err(_) => match from_json_slice(body_bytes) {
                Ok(json) => ErrorBody::Json(json),
                Err(error) => ErrorBody::NotJson {
                    bytes: Bytes::copy_from_slice(body_bytes),
                    deserialization_error: Arc::new(error),
                },
            },
        };

        error_body.into_error(status)
    }
}

/// The body of a Matrix API endpoint error.
#[derive(Debug, Clone)]
#[allow(clippy::exhaustive_enums)]
pub enum ErrorBody {
    /// A JSON body with the fields expected for Matrix endpoints errors.
    Standard(StandardErrorBody),

    /// A JSON body with an unexpected structure.
    Json(JsonValue),

    /// A response body that is not valid JSON.
    NotJson {
        /// The raw bytes of the response body.
        bytes: Bytes,

        /// The error from trying to deserialize the bytes as JSON.
        deserialization_error: Arc<serde_json::Error>,
    },
}

impl ErrorBody {
    /// Convert the ErrorBody into an Error by adding the http status code.
    ///
    /// This is equivalent to calling `Error::new(status_code, self)`.
    pub fn into_error(self, status_code: http::StatusCode) -> Error {
        Error { status_code, body: self }
    }
}

/// A JSON body with the fields expected for Matrix API endpoints errors.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct StandardErrorBody {
    /// A value which can be used to handle an error message.
    #[serde(flatten)]
    pub kind: ErrorKind,

    /// A human-readable error message, usually a sentence explaining what went wrong.
    #[serde(rename = "error")]
    pub message: String,
}

impl StandardErrorBody {
    /// Construct a new `StandardErrorBody` with the given kind and message.
    pub fn new(kind: ErrorKind, message: String) -> Self {
        Self { kind, message }
    }
}

/// An error when converting one of ruma's endpoint-specific request or response
/// types to the corresponding http type.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum IntoHttpError {
    /// Failed to add the authentication scheme to the request.
    #[error("failed to add authentication scheme: {0}")]
    Authentication(Box<dyn std::error::Error + Send + Sync + 'static>),

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
    #[error(
        "could not create any path variant for endpoint, as it was removed in version {}",
        .0.as_str().expect("no endpoint was removed in Matrix 1.0")
    )]
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

/// Extension trait for `FromHttpResponseError<Error>`.
pub trait FromHttpResponseErrorExt {
    /// If `self` is a server error in the `errcode` + `error` format expected
    /// for Matrix API endpoints, returns the error kind (`errcode`).
    fn error_kind(&self) -> Option<&ErrorKind>;
}

impl FromHttpResponseErrorExt for FromHttpResponseError<Error> {
    fn error_kind(&self) -> Option<&ErrorKind> {
        as_variant!(self, Self::Server)?.error_kind()
    }
}

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

    /// Deserialization of `multipart/mixed` response failed.
    #[error(transparent)]
    MultipartMixed(#[from] MultipartMixedDeserializationError),
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

    /// The given header failed to parse.
    #[error("invalid header: {0}")]
    InvalidHeader(Box<dyn std::error::Error + Send + Sync + 'static>),

    /// A header was received with a unexpected value.
    #[error(
        "The {header} header was received with an unexpected value, \
         expected {expected}, received {unexpected}"
    )]
    InvalidHeaderValue {
        /// The name of the header containing the invalid value.
        header: String,
        /// The value the header should have been set to.
        expected: String,
        /// The value we instead received and rejected.
        unexpected: String,
    },

    /// The `Content-Type` header for a `multipart/mixed` response is missing the `boundary`
    /// attribute.
    #[error(
        "The `Content-Type` header for a `multipart/mixed` response is missing the `boundary` attribute"
    )]
    MissingMultipartBoundary,
}

/// An error when deserializing a `multipart/mixed` response.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum MultipartMixedDeserializationError {
    /// There were not the number of body parts that were expected.
    #[error(
        "multipart/mixed response does not have enough body parts, \
         expected {expected}, found {found}"
    )]
    MissingBodyParts {
        /// The number of body parts expected in the response.
        expected: usize,
        /// The number of body parts found in the received response.
        found: usize,
    },

    /// The separator between the headers and the content of a body part is missing.
    #[error("multipart/mixed body part is missing separator between headers and content")]
    MissingBodyPartInnerSeparator,

    /// The separator between a header's name and value is missing.
    #[error("multipart/mixed body part header is missing separator between name and value")]
    MissingHeaderSeparator,

    /// A header failed to parse.
    #[error("invalid multipart/mixed header: {0}")]
    InvalidHeader(Box<dyn std::error::Error + Send + Sync + 'static>),
}

/// An error that happens when Ruma cannot understand a Matrix version.
#[derive(Debug)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct UnknownVersionError;

impl fmt::Display for UnknownVersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "version string was unknown")
    }
}

impl StdError for UnknownVersionError {}

/// An error that happens when an incorrect amount of arguments have been passed to [`PathBuilder`]
/// parts formatting.
///
/// [`PathBuilder`]: super::path_builder::PathBuilder
#[derive(Debug)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
