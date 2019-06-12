//! Crate ruma_api contains core types used to define the requests and responses for each endpoint
//! in the various [Matrix](https://matrix.org) API specifications.
//! These types can be shared by client and server code for all Matrix APIs.
//!
//! When implementing a new Matrix API, each endpoint has a type that implements `Endpoint`, plus
//! the necessary associated types.
//! An implementation of `Endpoint` contains all the information about the HTTP method, the path and
//! input parameters for requests, and the structure of a successful response.
//! Such types can then be used by client code to make requests, and by server code to fulfill
//! those requests.

#![deny(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    warnings
)]
#![warn(
    clippy::empty_line_after_outer_attr,
    clippy::expl_impl_clone_on_copy,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::match_same_arms,
    clippy::mem_forget,
    clippy::missing_docs_in_private_items,
    clippy::multiple_inherent_impl,
    clippy::mut_mut,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::single_match_else,
    clippy::unicode_not_nfc,
    clippy::use_self,
    clippy::used_underscore_binding,
    clippy::wrong_pub_self_convention,
    clippy::wrong_self_convention
)]

use std::{
    convert::TryInto,
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io,
};

use futures::future::FutureFrom;
use http::{self, Method, Request, Response, StatusCode};
use hyper::{self, Body};
use ruma_identifiers;
use serde_json;
use serde_urlencoded;

/// A Matrix API endpoint.
pub trait Endpoint<T = Body, U = Body> {
    /// Data needed to make a request to the endpoint.
    type Request: TryInto<Request<T>, Error = Error> + FutureFrom<Request<T>, Error = Error>;
    /// Data returned from the endpoint.
    type Response: FutureFrom<Response<U>, Error = Error> + TryInto<Response<U>>;

    /// Metadata about the endpoint.
    const METADATA: Metadata;
}

/// An error when converting an `Endpoint::Request` to a `http::Request` or a `http::Response` to
/// an `Endpoint::Response`.
#[derive(Debug)]
pub struct Error(pub(crate) InnerError);

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let message = match self.0 {
            InnerError::Http(_) => "An error converting to or from `http` types occurred.".into(),
            InnerError::Hyper(_) => "A Hyper error occurred.".into(),
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

#[derive(Debug)]
pub(crate) enum InnerError {
    /// An HTTP error.
    Http(http::Error),
    /// An Hyper error.
    Hyper(hyper::Error),
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

impl From<hyper::Error> for Error {
    fn from(error: hyper::Error) -> Self {
        Self(InnerError::Hyper(error))
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

        use futures::future::{err, ok, FutureFrom, FutureResult};
        use http::{
            header::CONTENT_TYPE, method::Method, Request as HttpRequest, Response as HttpResponse,
        };
        use ruma_identifiers::{RoomAliasId, RoomId};
        use serde::{de::IntoDeserializer, Deserialize, Serialize};
        use serde_json;
        use url::percent_encoding;

        use super::super::{Endpoint as ApiEndpoint, Error, Metadata};

        #[derive(Debug)]
        pub struct Endpoint;

        impl ApiEndpoint<Vec<u8>, Vec<u8>> for Endpoint {
            type Request = Request;
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

        /// A request to create a new room alias.
        #[derive(Debug)]
        pub struct Request {
            pub room_id: RoomId,         // body
            pub room_alias: RoomAliasId, // path
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct RequestBody {
            room_id: RoomId,
        }

        impl TryFrom<Request> for HttpRequest<Vec<u8>> {
            type Error = Error;

            fn try_from(request: Request) -> Result<HttpRequest<Vec<u8>>, Self::Error> {
                let metadata = Endpoint::METADATA;

                let path = metadata
                    .path
                    .to_string()
                    .replace(":room_alias", &request.room_alias.to_string());

                let request_body = RequestBody {
                    room_id: request.room_id,
                };

                let http_request = HttpRequest::builder()
                    .method(metadata.method)
                    .uri(path)
                    .body(serde_json::to_vec(&request_body).map_err(Error::from)?)?;

                Ok(http_request)
            }
        }

        impl FutureFrom<HttpRequest<Vec<u8>>> for Request {
            type Future = FutureResult<Self, Self::Error>;
            type Error = Error;

            fn future_from(request: HttpRequest<Vec<u8>>) -> Self::Future {
                FutureResult::from(Self::try_from(request))
            }
        }

        impl TryFrom<HttpRequest<Vec<u8>>> for Request {
            type Error = Error;

            fn try_from(request: HttpRequest<Vec<u8>>) -> Result<Request, Self::Error> {
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

        /// The response to a request to create a new room alias.
        #[derive(Clone, Copy, Debug)]
        pub struct Response;

        impl FutureFrom<HttpResponse<Vec<u8>>> for Response {
            type Future = FutureResult<Self, Self::Error>;
            type Error = Error;

            fn future_from(
                http_response: HttpResponse<Vec<u8>>,
            ) -> FutureResult<Self, Self::Error> {
                if http_response.status().is_success() {
                    ok(Response)
                } else {
                    err(http_response.status().clone().into())
                }
            }
        }

        impl TryFrom<Response> for HttpResponse<Vec<u8>> {
            type Error = Error;

            fn try_from(_response: Response) -> Result<HttpResponse<Vec<u8>>, Self::Error> {
                let response = HttpResponse::builder()
                    .header(CONTENT_TYPE, "application/json")
                    .body("{}".as_bytes().to_vec())
                    .unwrap();
                Ok(response)
            }
        }
    }
}
