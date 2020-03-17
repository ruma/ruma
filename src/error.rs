//! This module contains types for all kinds of errors that can occur when
//! converting between http requests / responses and ruma's representation of
//! matrix API requests / responses.

use std::fmt::{self, Display, Formatter};

// FIXME when `!` becomes stable use it
/// Default `ResponseError` for `ruma_api!` macro
#[derive(Clone, Copy, Debug)]
pub struct Void;

impl crate::EndpointError for Void {
    fn try_from_response(
        response: http::Response<Vec<u8>>,
    ) -> Result<Self, ResponseDeserializationError> {
        Err(ResponseDeserializationError::from_response(response))
    }
}
/// An error when converting one of ruma's endpoint-specific request or response
/// types to the corresponding http type.
#[derive(Debug)]
pub struct IntoHttpError(SerializationError);

#[doc(hidden)]
impl From<serde_json::Error> for IntoHttpError {
    fn from(err: serde_json::Error) -> Self {
        Self(SerializationError::Json(err))
    }
}

#[doc(hidden)]
impl From<serde_urlencoded::ser::Error> for IntoHttpError {
    fn from(err: serde_urlencoded::ser::Error) -> Self {
        Self(SerializationError::Query(err))
    }
}

impl Display for IntoHttpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.0 {
            SerializationError::Json(err) => write!(f, "JSON serialization failed: {}", err),
            SerializationError::Query(err) => {
                write!(f, "Query parameter serialization failed: {}", err)
            }
        }
    }
}

impl std::error::Error for IntoHttpError {}

/// An error when converting a http request to one of ruma's endpoint-specific
/// request types.
#[derive(Debug)]
#[non_exhaustive]
pub enum FromHttpRequestError {
    /// Deserialization failed
    Deserialization(RequestDeserializationError),
}

impl Display for FromHttpRequestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Deserialization(err) => write!(f, "deserialization failed: {}", err),
        }
    }
}

impl From<RequestDeserializationError> for FromHttpRequestError {
    fn from(err: RequestDeserializationError) -> Self {
        Self::Deserialization(err)
    }
}

impl std::error::Error for FromHttpRequestError {}

/// An error that occurred when trying to deserialize a request.
#[derive(Debug)]
pub struct RequestDeserializationError {
    inner: DeserializationError,
    http_request: http::Request<Vec<u8>>,
}

impl RequestDeserializationError {
    /// This method is public so it is accessible from `ruma_api!` generated
    /// code. It is not considered part of ruma-api's public API.
    #[doc(hidden)]
    pub fn new(
        inner: impl Into<DeserializationError>,
        http_request: http::Request<Vec<u8>>,
    ) -> Self {
        Self { inner: inner.into(), http_request }
    }
}

impl Display for RequestDeserializationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl std::error::Error for RequestDeserializationError {}

/// An error when converting a http response to one of ruma's endpoint-specific
/// response types.
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

/// An error that occurred when trying to deserialize a response.
#[derive(Debug)]
pub struct ResponseDeserializationError {
    inner: Option<DeserializationError>,
    http_response: http::Response<Vec<u8>>,
}

impl ResponseDeserializationError {
    /// This method is public so it is accessible from `ruma_api!` generated
    /// code. It is not considered part of ruma-api's public API.
    #[doc(hidden)]
    pub fn new(
        inner: impl Into<DeserializationError>,
        http_response: http::Response<Vec<u8>>,
    ) -> Self {
        Self { inner: Some(inner.into()), http_response }
    }

    /// This method is public so it is accessible from `ruma_api!` generated
    /// code. It is not considered part of ruma-api's public API.
    /// Creates an Error from a `http::Response`.
    #[doc(hidden)]
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

impl std::error::Error for ResponseDeserializationError {}

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

impl<E: std::error::Error> std::error::Error for ServerError<E> {}

#[derive(Debug)]
enum SerializationError {
    Json(serde_json::Error),
    Query(serde_urlencoded::ser::Error),
}

/// This type is public so it is accessible from `ruma_api!` generated code.
/// It is not considered part of ruma-api's public API.
#[doc(hidden)]
#[derive(Debug)]
pub enum DeserializationError {
    Utf8(std::str::Utf8Error),
    Json(serde_json::Error),
    Query(serde_urlencoded::de::Error),
    Ident(ruma_identifiers::Error),
    // String <> Enum conversion failed. This can currently only happen in path
    // segment deserialization
    Strum(strum::ParseError),
}

impl Display for DeserializationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DeserializationError::Utf8(err) => Display::fmt(err, f),
            DeserializationError::Json(err) => Display::fmt(err, f),
            DeserializationError::Query(err) => Display::fmt(err, f),
            DeserializationError::Ident(err) => Display::fmt(err, f),
            DeserializationError::Strum(err) => Display::fmt(err, f),
        }
    }
}

#[doc(hidden)]
impl From<std::str::Utf8Error> for DeserializationError {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Utf8(err)
    }
}

#[doc(hidden)]
impl From<serde_json::Error> for DeserializationError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

#[doc(hidden)]
impl From<serde_urlencoded::de::Error> for DeserializationError {
    fn from(err: serde_urlencoded::de::Error) -> Self {
        Self::Query(err)
    }
}

#[doc(hidden)]
impl From<ruma_identifiers::Error> for DeserializationError {
    fn from(err: ruma_identifiers::Error) -> Self {
        Self::Ident(err)
    }
}

#[doc(hidden)]
impl From<strum::ParseError> for DeserializationError {
    fn from(err: strum::ParseError) -> Self {
        Self::Strum(err)
    }
}

#[doc(hidden)]
impl From<std::convert::Infallible> for DeserializationError {
    fn from(err: std::convert::Infallible) -> Self {
        match err {}
    }
}
