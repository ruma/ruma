//! This module contains types for all kinds of errors that can occur when
//! converting between http requests / responses and ruma's representation of
//! matrix API requests / responses.

use std::{error::Error as StdError, fmt};

use bytes::BufMut;
use serde_json::{from_slice as from_json_slice, Value as JsonValue};
use thiserror::Error;

use crate::{EndpointError, MatrixVersion, OutgoingResponse};

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
            .body(ruma_serde::json_to_buf(&self.body)?)
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
    #[error(
        "This endpoint has to be converted to http::Request using \
         try_into_authenticated_http_request"
    )]
    NeedsAuthentication,

    /// Tried to create a request with an old enough version, for which no unstable endpoint
    /// exists.
    ///
    /// This is also a fallback error for if the version is too new for this endpoint.
    #[error("Tried to fall back to unstable path, but it did not exist for this endpoint.")]
    NoUnstablePath,

    /// Tried to create a request with a [`MatrixVersion`] in which this endpoint was removed.
    #[error("Could not create any path variant for endpoint, as it was removed in version {0}")]
    EndpointRemoved(MatrixVersion),

    /// JSON serialization failed.
    #[error("JSON serialization failed: {0}")]
    Json(#[from] serde_json::Error),

    /// Query parameter serialization failed.
    #[error("Query parameter serialization failed: {0}")]
    Query(#[from] ruma_serde::urlencoded::ser::Error),

    /// Header serialization failed.
    #[error("Header serialization failed: {0}")]
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
    Http(ServerError<E>),
}

impl<E: fmt::Display> fmt::Display for FromHttpResponseError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Deserialization(err) => write!(f, "deserialization failed: {}", err),
            Self::Http(err) => write!(f, "the server returned an error: {}", err),
        }
    }
}

impl<E> From<ServerError<E>> for FromHttpResponseError<E> {
    fn from(err: ServerError<E>) -> Self {
        Self::Http(err)
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
    #[error("{0}")]
    Utf8(#[from] std::str::Utf8Error),

    /// JSON deserialization failed.
    #[error("{0}")]
    Json(#[from] serde_json::Error),

    /// Query parameter deserialization failed.
    #[error("{0}")]
    Query(#[from] ruma_serde::urlencoded::de::Error),

    /// Got an invalid identifier.
    #[error("{0}")]
    Ident(#[from] ruma_identifiers::Error),

    /// Header value deserialization failed.
    #[error("{0}")]
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
    #[error("Missing header `{0}`")]
    MissingHeader(String),
}

/// An error that happens when Ruma cannot understand a Matrix version.
#[derive(Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct UnknownVersionError;

impl fmt::Display for UnknownVersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Version string was unknown.")
    }
}

impl StdError for UnknownVersionError {}
