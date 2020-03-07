//! This module contains types for all kinds of errors that can occur when
//! converting between http requests / responses and ruma's representation of
//! matrix API requests / responses.

use std::fmt::{self, Display, Formatter};

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
pub enum FromHttpResponseError {
    /// Deserialization failed
    Deserialization(ResponseDeserializationError),
    /// The server returned a non-success status
    Http(ServerError),
}

impl Display for FromHttpResponseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Deserialization(err) => write!(f, "deserialization failed: {}", err),
            Self::Http(err) => write!(f, "the server returned an error: {}", err),
        }
    }
}

impl From<ServerError> for FromHttpResponseError {
    fn from(err: ServerError) -> Self {
        Self::Http(err)
    }
}

impl From<ResponseDeserializationError> for FromHttpResponseError {
    fn from(err: ResponseDeserializationError) -> Self {
        Self::Deserialization(err)
    }
}

/// An error that occurred when trying to deserialize a response.
#[derive(Debug)]
pub struct ResponseDeserializationError {
    inner: DeserializationError,
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
        Self { inner: inner.into(), http_response }
    }
}

impl Display for ResponseDeserializationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl std::error::Error for ResponseDeserializationError {}

/// An error was reported by the server (HTTP status code 4xx or 5xx)
#[derive(Debug)]
pub struct ServerError {
    http_response: http::Response<Vec<u8>>,
}

impl ServerError {
    /// This method is public so it is accessible from `ruma_api!` generated
    /// code. It is not considered part of ruma-api's public API.
    #[doc(hidden)]
    pub fn new(http_response: http::Response<Vec<u8>>) -> Self {
        Self { http_response }
    }

    /// Get the HTTP response without parsing its contents.
    pub fn into_raw_reponse(self) -> http::Response<Vec<u8>> {
        self.http_response
    }
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.http_response.status().canonical_reason() {
            Some(reason) => {
                write!(f, "HTTP status {} {}", self.http_response.status().as_str(), reason)
            }
            None => write!(f, "HTTP status {}", self.http_response.status().as_str()),
        }
    }
}

impl std::error::Error for ServerError {}

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
