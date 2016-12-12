//! Crate ruma_client_api contains serializable types for the requests and responses for each
//! endpoint in the [Matrix](https://matrix.org/) client API specification. These types can be
//! shared by client and server code.

#![feature(proc_macro)]

extern crate ruma_events;
extern crate ruma_identifiers;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

use serde::{Deserialize, Serialize};

/// Endpoints for the r0.x.x versions of the client API specification.
pub mod r0 {
    pub mod account;
    pub mod alias;
    pub mod config;
    pub mod contact;
    pub mod context;
    pub mod directory;
    pub mod filter;
    pub mod media;
    pub mod membership;
    pub mod presence;
    pub mod profile;
    pub mod push;
    pub mod receipt;
    pub mod redact;
    pub mod room;
    pub mod search;
    pub mod send;
    pub mod server;
    pub mod session;
    pub mod sync;
    pub mod tag;
    pub mod typing;
    pub mod voip;
}

/// GET /_matrix/client/versions
pub mod supported_versions;

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
    type QueryParams;

    /// The body of the response.
    type Response: Deserialize + Serialize;

    /// Returns the HTTP method used by this endpoint.
    fn method() -> Method;

    /// Generates the path component of the URL for this endpoint using the supplied parameters.
    fn request_path(params: Self::PathParams) -> String;

    /// Generates a generic path component of the URL for this endpoint, suitable for `Router` from
    /// the router crate.
    fn router_path() -> String;
}
