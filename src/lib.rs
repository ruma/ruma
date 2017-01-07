//! Crate ruma_api contains core types used to define the requests and responses for each endpoint
//! in the various [Matrix](https://matrix.org) API specifications.
//! These types can be shared by client and server code for all Matrix APIs.
//!
//! When implementing a new Matrix API, each endpoint have a type that implements `Endpoint`, plus
//! any necessary associated types.
//! An implementation of `Endpoint` contains all the information about the HTTP method, the path and
//! input parameters for requests, and the structure of a successful response.
//! Such types can then be used by client code to make requests, and by server code to fulfill
//! those requests.
//!
//! # Example
//!
//! ```rust,no_run
//! # #![feature(proc_macro)]
//! #
//! # extern crate ruma_api;
//! # extern crate ruma_identifiers;
//! # #[macro_use]
//! # extern crate serde_derive;
//! #
//! # fn main() {
//! /// PUT /_matrix/client/r0/directory/room/:room_alias
//! pub mod create {
//!     use ruma_api;
//!     use ruma_identifiers::{RoomAliasId, RoomId};
//!
//!     /// This API endpoint's body parameters.
//!     #[derive(Clone, Debug, Deserialize, Serialize)]
//!     pub struct BodyParams {
//!         pub room_id: RoomId,
//!     }
//!
//!     /// This API endpoint's path parameters.
//!     #[derive(Clone, Debug)]
//!     pub struct PathParams {
//!         pub room_alias: RoomAliasId,
//!     }
//!
//!     /// Details about this API endpoint.
//!     pub struct Endpoint;
//!
//!     impl ruma_api::Endpoint for Endpoint {
//!         type BodyParams = BodyParams;
//!         type PathParams = PathParams;
//!         type QueryParams = ();
//!         type Response = ();
//!
//!         fn method() -> ruma_api::Method {
//!             ruma_api::Method::Put
//!         }
//!
//!         fn request_path(params: Self::PathParams) -> String {
//!             format!("/_matrix/client/r0/directory/room/{}", params.room_alias)
//!         }
//!
//!         fn router_path() -> &'static str {
//!             "/_matrix/client/r0/directory/room/:room_alias"
//!         }
//!
//!         fn name() -> &'static str {
//!             "room_directory"
//!         }
//!
//!         fn description() -> &'static str {
//!             "Matrix implementation of room directory."
//!         }
//!
//!         fn requires_authentication() -> bool {
//!             true
//!         }
//!
//!         fn rate_limited() -> bool {
//!             false
//!         }
//!     }
//! }
//! # }

#![deny(missing_docs)]

extern crate serde;

use serde::{Deserialize, Serialize};

/// HTTP request methods used in Matrix APIs.
#[derive(Clone, Copy, Debug)]
pub enum Method {
    /// DELETE
    Delete,
    /// GET
    Get,
    /// POST
    Post,
    /// PUT
    Put,
}

/// An API endpoint.
pub trait Endpoint {
    /// Request parameters supplied via the body of the HTTP request.
    type BodyParams: Deserialize + Serialize;

    /// Request parameters supplied via the URL's path.
    type PathParams;

    /// Parameters supplied via the URL's query string.
    type QueryParams: Deserialize + Serialize;

    /// The body of the response.
    type Response: Deserialize + Serialize;

    /// Returns the HTTP method used by this endpoint.
    fn method() -> Method;

    /// Generates the path component of the URL for this endpoint using the supplied parameters.
    fn request_path(params: Self::PathParams) -> String;

    /// Generates a generic path component of the URL for this endpoint, suitable for `Router` from
    /// the router crate.
    fn router_path() -> &'static str;

    /// A unique identifier for this endpoint, suitable for `Router` from the router crate.
    fn name() -> &'static str;

    /// A human-readable description of the endpoint.
    fn description() -> &'static str;

    /// Whether or not this endpoint requires an authenticated user.
    fn requires_authentication() -> bool;

    /// Whether or not this endpoint is rate limited.
    fn rate_limited() -> bool;
}
