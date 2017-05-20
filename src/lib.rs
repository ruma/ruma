//! Crate ruma_client_api contains serializable types for the requests and responses for each
//! endpoint in the [Matrix](https://matrix.org/) client API specification. These types can be
//! shared by client and server code.

#![deny(missing_debug_implementations)]
#![feature(associated_consts, proc_macro, try_from)]
#![warn(missing_docs)]

extern crate futures;
extern crate hyper;
extern crate ruma_api;
extern crate ruma_api_macros;
extern crate ruma_events;
extern crate ruma_identifiers;
extern crate ruma_signatures;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

/// Endpoints for the r0.x.x versions of the client API specification.
pub mod r0 {
    pub mod account;
    pub mod alias;
    pub mod config;
    pub mod contact;
    pub mod context;
//     pub mod directory;
//     pub mod filter;
//     pub mod media;
//     pub mod membership;
//     pub mod presence;
//     pub mod profile;
//     pub mod push;
//     pub mod receipt;
//     pub mod redact;
//     pub mod room;
//     pub mod search;
//     pub mod send;
//     pub mod server;
//     pub mod session;
//     pub mod sync;
//     pub mod tag;
//     pub mod typing;
//     pub mod voip;
}

pub mod unversioned;
