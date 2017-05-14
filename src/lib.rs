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

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![feature(associated_consts, try_from)]

extern crate hyper;
#[cfg(test)] extern crate ruma_identifiers;
#[cfg(test)] extern crate serde;
#[cfg(test)] #[macro_use] extern crate serde_derive;
extern crate serde_json;

use std::convert::{TryFrom, TryInto};
use std::io;

use hyper::{Method, Request, Response, StatusCode};
use hyper::error::UriError;

/// A Matrix API endpoint.
pub trait Endpoint {
    /// Data needed to make a request to the endpoint.
    type Request: TryInto<Request, Error = Error>;
    /// Data returned from the endpoint.
    type Response: TryFrom<Response, Error = Error>;

    /// Metadata about the endpoint.
    const METADATA: Metadata;
}

/// An error when converting an `Endpoint::Request` to a `hyper::Request` or a `hyper::Response` to
/// an `Endpoint::Response`.
#[derive(Debug)]
pub enum Error {
    /// A Hyper error.
    Hyper(hyper::Error),
    /// A I/O error.
    Io(io::Error),
    /// A Serde JSON error.
    SerdeJson(serde_json::Error),
    /// An HTTP status code indicating error.
    StatusCode(StatusCode),
    /// A Uri error.
    Uri(UriError),
}

impl From<hyper::Error> for Error {
    fn from(error: hyper::Error) -> Self {
        Error::Hyper(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::SerdeJson(error)
    }
}

impl From<UriError> for Error {
    fn from(error: UriError) -> Self {
        Error::Uri(error)
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

        use hyper::{Method, Request as HyperRequest, Response as HyperResponse, StatusCode};
        use ruma_identifiers::{RoomAliasId, RoomId};
        use serde_json;

        use super::super::{Endpoint as ApiEndpoint, Error, Metadata};

        #[derive(Debug)]
        pub struct Endpoint;

        impl ApiEndpoint for Endpoint {
            type Request = Request;
            type Response = Response;

            const METADATA: Metadata = Metadata {
                description: "Add an alias to a room.",
                method: Method::Put,
                name: "create_alias",
                path: "/_matrix/client/r0/directory/room/:room_alias",
                rate_limited: false,
                requires_authentication: true,
            };

        }

        /// A request to create a new room alias.
        #[derive(Debug)]
        pub struct Request {
            pub room_id: RoomId, // body
            pub room_alias: RoomAliasId, // path
        }

        #[derive(Debug, Serialize)]
        struct RequestBody {
            room_id: RoomId,
        }

        impl TryFrom<Request> for HyperRequest {
            type Error = Error;

            fn try_from(request: Request) -> Result<HyperRequest, Self::Error> {
                let metadata = Endpoint::METADATA;

                let path = metadata.path
                    .to_string()
                    .replace(":room_alias", &request.room_alias.to_string());

                let mut hyper_request = HyperRequest::new(
                    metadata.method,
                    path.parse().map_err(Error::from)?,
                );

                let request_body = RequestBody {
                    room_id: request.room_id,
                };

                hyper_request.set_body(serde_json::to_vec(&request_body).map_err(Error::from)?);

                Ok(hyper_request)
            }
        }

        /// The response to a request to create a new room alias.
        pub struct Response;

        impl TryFrom<HyperResponse> for Response {
            type Error = Error;

            fn try_from(hyper_response: HyperResponse) -> Result<Response, Self::Error> {
                if hyper_response.status() == StatusCode::Ok {
                    Ok(Response)
                } else {
                    Err(Error::StatusCode(hyper_response.status().clone()))
                }
            }
        }
    }
}
