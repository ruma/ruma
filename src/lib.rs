//! Crate `ruma_api` contains core types used to define the requests and responses for each endpoint
//! in the various [Matrix](https://matrix.org) API specifications.
//! These types can be shared by client and server code for all Matrix APIs.
//!
//! When implementing a new Matrix API, each endpoint has a request type which implements
//! `Endpoint`, and a response type connected via an associated type.
//!
//! An implementation of `Endpoint` contains all the information about the HTTP method, the path and
//! input parameters for requests, and the structure of a successful response.
//! Such types can then be used by client code to make requests, and by server code to fulfill
//! those requests.

#![warn(rust_2018_idioms)]
#![deny(missing_copy_implementations, missing_debug_implementations, missing_docs)]
// Since we support Rust 1.34.2, we can't apply this suggestion yet
#![allow(clippy::use_self)]

use std::{
    convert::{TryFrom, TryInto},
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io,
};

use http::{self, Method, StatusCode};
use ruma_identifiers;
use serde_json;
use serde_urlencoded;

#[cfg(feature = "with-ruma-api-macros")]
pub use ruma_api_macros::ruma_api;

#[cfg(feature = "with-ruma-api-macros")]
#[doc(hidden)]
/// This module is used to support the generated code from ruma-api-macros.
/// It is not considered part of ruma-api's public API.
pub mod exports {
    pub use http;
    pub use serde;
    pub use serde_json;
    pub use serde_urlencoded;
    pub use url;
}

/// A Matrix API endpoint.
///
/// The type implementing this trait contains any data needed to make a request to the endpoint.
pub trait Endpoint: TryInto<http::Request<Vec<u8>>, Error = Error> {
    /// Data returned in a successful response from the endpoint.
    type Response: TryFrom<http::Response<Vec<u8>>, Error = Error>;

    /// Metadata about the endpoint.
    const METADATA: Metadata;
}

/// An error when converting an `Endpoint` request or response to the corresponding type from the
/// `http` crate.
#[derive(Debug)]
pub struct Error(pub(crate) InnerError);

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let message = match self.0 {
            InnerError::Http(_) => "An error converting to or from `http` types occurred.".into(),
            InnerError::Io(_) => "An I/O error occurred.".into(),
            InnerError::SerdeJson(_) => "A JSON error occurred.".into(),
            InnerError::SerdeUrlEncodedDe(_) => {
                "A URL encoding deserialization error occurred.".into()
            }
            InnerError::SerdeUrlEncodedSer(_) => {
                "A URL encoding serialization error occurred.".into()
            }
            InnerError::RumaIdentifiers(_) => "A ruma-identifiers error occurred.".into(),
            InnerError::StatusCode(code) => format!("A HTTP {} error occurred.", code),
        };

        write!(f, "{}", message)
    }
}

impl StdError for Error {}

/// Internal representation of errors.
#[derive(Debug)]
pub(crate) enum InnerError {
    /// An HTTP error.
    Http(http::Error),

    /// A I/O error.
    Io(io::Error),

    /// A Serde JSON error.
    SerdeJson(serde_json::Error),

    /// A Serde URL decoding error.
    SerdeUrlEncodedDe(serde_urlencoded::de::Error),

    /// A Serde URL encoding error.
    SerdeUrlEncodedSer(serde_urlencoded::ser::Error),

    /// A Ruma Identitifiers error.
    RumaIdentifiers(ruma_identifiers::Error),

    /// An HTTP status code indicating error.
    StatusCode(StatusCode),
}

impl From<http::Error> for Error {
    fn from(error: http::Error) -> Self {
        Self(InnerError::Http(error))
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self(InnerError::Io(error))
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self(InnerError::SerdeJson(error))
    }
}

impl From<serde_urlencoded::de::Error> for Error {
    fn from(error: serde_urlencoded::de::Error) -> Self {
        Self(InnerError::SerdeUrlEncodedDe(error))
    }
}

impl From<serde_urlencoded::ser::Error> for Error {
    fn from(error: serde_urlencoded::ser::Error) -> Self {
        Self(InnerError::SerdeUrlEncodedSer(error))
    }
}

impl From<ruma_identifiers::Error> for Error {
    fn from(error: ruma_identifiers::Error) -> Self {
        Self(InnerError::RumaIdentifiers(error))
    }
}

impl From<StatusCode> for Error {
    fn from(error: StatusCode) -> Self {
        Self(InnerError::StatusCode(error))
    }
}

/// Metadata about an API endpoint.
#[derive(Clone, Debug)]
pub struct Metadata {
    /// A human-readable description of the endpoint.
    pub description: &'static str,

    /// The HTTP method used by this endpoint.
    pub method: Method,

    /// A unique identifier for this endpoint.
    pub name: &'static str,

    /// The path of this endpoint's URL, with variable names where path parameters should be filled
    /// in during a request.
    pub path: &'static str,

    /// Whether or not this endpoint is rate limited by the server.
    pub rate_limited: bool,

    /// Whether or not the server requires an authenticated user for this endpoint.
    pub requires_authentication: bool,
}

#[cfg(test)]
mod tests {
    /// PUT /_matrix/client/r0/directory/room/:room_alias
    pub mod create {
        use std::convert::TryFrom;

        use http::{self, method::Method};
        use ruma_identifiers::{RoomAliasId, RoomId};
        use serde::{Deserialize, Serialize};
        use serde_json;

        use crate::{Endpoint, Error, Metadata};

        /// A request to create a new room alias.
        #[derive(Debug)]
        pub struct Request {
            pub room_id: RoomId,         // body
            pub room_alias: RoomAliasId, // path
        }

        impl Endpoint for Request {
            type Response = Response;

            const METADATA: Metadata = Metadata {
                description: "Add an alias to a room.",
                method: Method::PUT,
                name: "create_alias",
                path: "/_matrix/client/r0/directory/room/:room_alias",
                rate_limited: false,
                requires_authentication: true,
            };
        }

        impl TryFrom<Request> for http::Request<Vec<u8>> {
            type Error = Error;

            fn try_from(request: Request) -> Result<http::Request<Vec<u8>>, Self::Error> {
                let metadata = Request::METADATA;

                let path = metadata
                    .path
                    .to_string()
                    .replace(":room_alias", &request.room_alias.to_string());

                let request_body = RequestBody { room_id: request.room_id };

                let http_request = http::Request::builder()
                    .method(metadata.method)
                    .uri(path)
                    .body(serde_json::to_vec(&request_body).map_err(Error::from)?)?;

                Ok(http_request)
            }
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct RequestBody {
            room_id: RoomId,
        }

        /// The response to a request to create a new room alias.
        #[derive(Clone, Copy, Debug)]
        pub struct Response;

        impl TryFrom<http::Response<Vec<u8>>> for Response {
            type Error = Error;

            fn try_from(http_response: http::Response<Vec<u8>>) -> Result<Response, Self::Error> {
                if http_response.status().is_success() {
                    Ok(Response)
                } else {
                    Err(http_response.status().into())
                }
            }
        }
    }
}
