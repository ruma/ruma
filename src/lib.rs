//! Crate ruma_client_api contains serializable types for the requests and responses for each
//! endpoint in the [Matrix](https://matrix.org/) client API specification. These types can be
//! shared by client and server code.

#![deny(missing_debug_implementations, missing_docs)]
#![feature(try_from)]

use futures;
use http;
use hyper;
use ruma_api;
use serde;
use serde_json;
use serde_urlencoded;
use url;

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

pub mod unversioned;
