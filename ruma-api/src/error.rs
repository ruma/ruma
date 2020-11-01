//! This module contains types for all kinds of errors that can occur when
//! converting between http requests / responses and ruma's representation of
//! matrix API requests / responses.

use std::{
    error::Error as StdError,
    fmt::{self, Debug, Display, Formatter},
};

use thiserror::Error;

use crate::EndpointError;

// FIXME when `!` becomes stable use it
/// Default `EndpointError` for `ruma_api!` macro
#[derive(Clone, Copy, Debug)]
pub enum Void {}

impl EndpointError for Void {
    fn try_from_response(
        response: http::Response<Vec<u8>>,
    ) -> Result<Self, ResponseDeserializationError> {
        Err(ResponseDeserializationError::from_response(response))
    }
}

impl Display for Void {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}

impl StdError for Void {}

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
    Deserialization(#[from] RequestDeserializationError),
}

/// An error that occurred when trying to deserialize a request.
#[derive(Debug, Error)]
#[error("{inner}")]
pub struct RequestDeserializationError {
    inner: DeserializationError,
    http_request: http::Request<Vec<u8>>,
}

impl RequestDeserializationError {
    /// Creates a new `RequestDeserializationError` from the given deserialization error and http
    /// request.
    pub fn new(
        inner: impl Into<DeserializationError>,
        http_request: http::Request<Vec<u8>>,
    ) -> Self {
        Self { inner: inner.into(), http_request }
    }
}

/// An error when converting a http response to one of Ruma's endpoint-specific response types.
#[derive(Debug)]
#[non_exhaustive]
pub enum FromHttpResponseError<E> {
    /// Deserialization failed
    Deserialization(ResponseDeserializationError),

    /// The server returned a non-success status
    Http(ServerError<E>),
}

impl<E: Display> Display for FromHttpResponseError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

impl<E> From<ResponseDeserializationError> for FromHttpResponseError<E> {
    fn from(err: ResponseDeserializationError) -> Self {
        Self::Deserialization(err)
    }
}

impl<E: StdError> StdError for FromHttpResponseError<E> {}

/// An error that occurred when trying to deserialize a response.
#[derive(Debug)]
pub struct ResponseDeserializationError {
    inner: Option<DeserializationError>,
    http_response: http::Response<Vec<u8>>,
}

impl ResponseDeserializationError {
    /// Creates a new `ResponseDeserializationError` from the given deserialization error and http
    /// response.
    pub fn new(
        inner: impl Into<DeserializationError>,
        http_response: http::Response<Vec<u8>>,
    ) -> Self {
        Self { inner: Some(inner.into()), http_response }
    }

    /// Creates a new `ResponseDeserializationError` without an inner deserialization error.
    pub fn from_response(http_response: http::Response<Vec<u8>>) -> Self {
        Self { http_response, inner: None }
    }
}

impl Display for ResponseDeserializationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(ref inner) = self.inner {
            Display::fmt(inner, f)
        } else {
            Display::fmt("deserialization error, no error specified", f)
        }
    }
}

impl StdError for ResponseDeserializationError {}

/// An error was reported by the server (HTTP status code 4xx or 5xx)
#[derive(Debug)]
pub enum ServerError<E> {
    /// An error that is expected to happen under certain circumstances and
    /// that has a well-defined structure
    Known(E),

    /// An error of unexpected type of structure
    Unknown(ResponseDeserializationError),
}

impl<E: Display> Display for ServerError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::Known(e) => Display::fmt(e, f),
            ServerError::Unknown(res_err) => Display::fmt(res_err, f),
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
    Header(#[from] http::header::ToStrError),
}

impl From<std::convert::Infallible> for DeserializationError {
    fn from(err: std::convert::Infallible) -> Self {
        match err {}
    }
}
