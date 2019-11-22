//! Crate ruma_client_api contains serializable types for the requests and responses for each
//! endpoint in the [Matrix](https://matrix.org/) client API specification. These types can be
//! shared by client and server code.

#![deny(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs
)]
// Since we support Rust 1.36.0, we can't apply this suggestion yet
#![allow(clippy::use_self)]

pub mod r0;
pub mod unversioned;
