//! Crate ruma_client_api contains serializable types for the requests and responses for each
//! endpoint in the [Matrix](https://matrix.org/) client API specification. These types can be
//! shared by client and server code.

#![warn(missing_debug_implementations, missing_docs)]

pub mod error;
pub mod r0;
pub mod unversioned;

pub use error::Error;
