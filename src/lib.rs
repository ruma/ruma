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
// Since we support Rust 1.36.0, we can't apply this suggestion yet
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

/// Generates a `ruma_api::Endpoint` from a concise definition.
///
/// The macro expects the following structure as input:
///
/// ```text
/// ruma_api! {
///     metadata {
///         description: &'static str,
///         method: http::Method,
///         name: &'static str,
///         path: &'static str,
///         rate_limited: bool,
///         requires_authentication: bool,
///     }
///
///     request {
///         // Struct fields for each piece of data required
///         // to make a request to this API endpoint.
///     }
///
///     response {
///         // Struct fields for each piece of data expected
///         // in the response from this API endpoint.
///     }
/// }
/// ```
///
/// This will generate a `ruma_api::Metadata` value to be used for the `ruma_api::Endpoint`'s
/// associated constant, single `Request` and `Response` structs, and the necessary trait
/// implementations to convert the request into a `http::Request` and to create a response from a
/// `http::Response` and vice versa.
///
/// The details of each of the three sections of the macros are documented below.
///
/// ## Metadata
///
/// *   `description`: A short description of what the endpoint does.
/// *   `method`: The HTTP method used for requests to the endpoint.
///     It's not necessary to import `http::Method`'s associated constants. Just write
///     the value as if it was imported, e.g. `GET`.
/// *   `name`: A unique name for the endpoint.
///     Generally this will be the same as the containing module.
/// *   `path`: The path component of the URL for the endpoint, e.g. "/foo/bar".
///     Components of the path that are parameterized can indicate a varible by using a Rust
///     identifier prefixed with a colon, e.g. `/foo/:some_parameter`.
///     A corresponding query string parameter will be expected in the request struct (see below
///     for details).
/// *   `rate_limited`: Whether or not the endpoint enforces rate limiting on requests.
/// *   `requires_authentication`: Whether or not the endpoint requires a valid access token.
///
/// ## Request
///
/// The request block contains normal struct field definitions.
/// Doc comments and attributes are allowed as normal.
/// There are also a few special attributes available to control how the struct is converted into a
/// `http::Request`:
///
/// *   `#[ruma_api(header = HEADER_NAME)]`: Fields with this attribute will be treated as HTTP
///     headers on the request.
///     The value must implement `AsRef<str>`.
///     Generally this is a `String`.
///     The attribute value shown above as `HEADER_NAME` must be a header name constant from
///     `http::header`, e.g. `CONTENT_TYPE`.
/// *   `#[ruma_api(path)]`: Fields with this attribute will be inserted into the matching path
///     component of the request URL.
/// *   `#[ruma_api(query)]`: Fields with this attribute will be inserting into the URL's query
///     string.
/// *   `#[ruma_api(query_map)]`: Instead of individual query fields, one query_map field, of any
///     type that implements `IntoIterator<Item = (String, String)>` (e.g.
///     `HashMap<String, String>`, can be used for cases where an endpoint supports arbitrary query
///     parameters.
///
/// Any field that does not include one of these attributes will be part of the request's JSON
/// body.
///
/// ## Response
///
/// Like the request block, the response block consists of normal struct field definitions.
/// Doc comments and attributes are allowed as normal.
/// There is also a special attribute available to control how the struct is created from a
/// `http::Request`:
///
/// *   `#[ruma_api(header = HEADER_NAME)]`: Fields with this attribute will be treated as HTTP
///     headers on the response.
///     The value must implement `AsRef<str>`.
///     Generally this is a `String`.
///     The attribute value shown above as `HEADER_NAME` must be a header name constant from
///     `http::header`, e.g. `CONTENT_TYPE`.
///
/// Any field that does not include the above attribute will be expected in the response's JSON
/// body.
///
/// ## Newtype bodies
///
/// Both the request and response block also support "newtype bodies" by using the
/// `#[ruma_api(body)]` attribute on a field. If present on a field, the entire request or response
/// body will be treated as the value of the field. This allows you to treat the entire request or
/// response body as a specific type, rather than a JSON object with named fields. Only one field in
/// each struct can be marked with this attribute. It is an error to have a newtype body field and
/// normal body fields within the same struct.
///
/// # Examples
///
/// ```
/// pub mod some_endpoint {
///     use ruma_api_macros::ruma_api;
///
///     ruma_api! {
///         metadata {
///             description: "Does something.",
///             method: POST,
///             name: "some_endpoint",
///             path: "/_matrix/some/endpoint/:baz",
///             rate_limited: false,
///             requires_authentication: false,
///         }
///
///         request {
///             pub foo: String,
///
///             #[ruma_api(header = CONTENT_TYPE)]
///             pub content_type: String,
///
///             #[ruma_api(query)]
///             pub bar: String,
///
///             #[ruma_api(path)]
///             pub baz: String,
///         }
///
///         response {
///             #[ruma_api(header = CONTENT_TYPE)]
///             pub content_type: String,
///
///             pub value: String,
///         }
///     }
/// }
///
/// pub mod newtype_body_endpoint {
///     use ruma_api_macros::ruma_api;
///     use serde::{Deserialize, Serialize};
///
///     #[derive(Clone, Debug, Deserialize, Serialize)]
///     pub struct MyCustomType {
///         pub foo: String,
///     }
///
///     ruma_api! {
///         metadata {
///             description: "Does something.",
///             method: PUT,
///             name: "newtype_body_endpoint",
///             path: "/_matrix/some/newtype/body/endpoint",
///             rate_limited: false,
///             requires_authentication: false,
///         }
///
///         request {
///             #[ruma_api(body)]
///             pub file: Vec<u8>,
///         }
///
///         response {
///             #[ruma_api(body)]
///             pub my_custom_type: MyCustomType,
///         }
///     }
/// }
/// ```
#[cfg(feature = "with-ruma-api-macros")]
pub use ruma_api_macros::ruma_api;

#[cfg(feature = "with-ruma-api-macros")]
pub use ruma_api_macros::SendRecv;

#[cfg(feature = "with-ruma-api-macros")]
#[doc(hidden)]
/// This module is used to support the generated code from ruma-api-macros.
/// It is not considered part of ruma-api's public API.
pub mod exports {
    pub use http;
    pub use percent_encoding;
    pub use serde;
    pub use serde_json;
    pub use serde_urlencoded;
    pub use url;
}

/// A type that can be sent as well as received. Types that implement this trait have a
/// corresponding 'Incoming' type, which is either just `Self`, or another type that has the same
/// fields with some types exchanged by ones that allow fallible deserialization, e.g. `EventResult`
/// from ruma_events.
pub trait SendRecv {
    /// The 'Incoming' variant of `Self`.
    type Incoming;
}

/// A Matrix API endpoint.
///
/// The type implementing this trait contains any data needed to make a request to the endpoint.
pub trait Endpoint: SendRecv + TryInto<http::Request<Vec<u8>>, Error = Error>
where
    <Self as SendRecv>::Incoming: TryFrom<http::Request<Vec<u8>>, Error = Error>,
    <Self::Response as SendRecv>::Incoming: TryFrom<http::Response<Vec<u8>>, Error = Error>,
{
    /// Data returned in a successful response from the endpoint.
    type Response: SendRecv + TryInto<http::Response<Vec<u8>>, Error = Error>;

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

        use http::{self, header::CONTENT_TYPE, method::Method};
        use percent_encoding;
        use ruma_identifiers::{RoomAliasId, RoomId};
        use serde::{de::IntoDeserializer, Deserialize, Serialize};
        use serde_json;

        use crate::{Endpoint, Error, Metadata, SendRecv};

        /// A request to create a new room alias.
        #[derive(Debug)]
        pub struct Request {
            pub room_id: RoomId,         // body
            pub room_alias: RoomAliasId, // path
        }

        impl SendRecv for Request {
            type Incoming = Self;
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

        impl TryFrom<http::Request<Vec<u8>>> for Request {
            type Error = Error;

            fn try_from(request: http::Request<Vec<u8>>) -> Result<Self, Self::Error> {
                let request_body: RequestBody =
                    ::serde_json::from_slice(request.body().as_slice())?;
                let path_segments: Vec<&str> = request.uri().path()[1..].split('/').collect();
                Ok(Request {
                    room_id: request_body.room_id,
                    room_alias: {
                        let segment = path_segments.get(5).unwrap().as_bytes();
                        let decoded = percent_encoding::percent_decode(segment).decode_utf8_lossy();
                        RoomAliasId::deserialize(decoded.into_deserializer())
                            .map_err(|e: serde_json::error::Error| e)?
                    },
                })
            }
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct RequestBody {
            room_id: RoomId,
        }

        /// The response to a request to create a new room alias.
        #[derive(Clone, Copy, Debug)]
        pub struct Response;

        impl SendRecv for Response {
            type Incoming = Self;
        }

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

        impl TryFrom<Response> for http::Response<Vec<u8>> {
            type Error = Error;

            fn try_from(_: Response) -> Result<http::Response<Vec<u8>>, Self::Error> {
                let response = http::Response::builder()
                    .header(CONTENT_TYPE, "application/json")
                    .body(b"{}".to_vec())
                    .unwrap();

                Ok(response)
            }
        }
    }
}
