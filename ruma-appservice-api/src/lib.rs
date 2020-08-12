//! Crate ruma_appservice_api contains serializable types for the requests and responses for each
//! endpoint in the [Matrix](https://matrix.org/) application service API specification. These
//! types can be shared by application service and server code.
#![warn(missing_copy_implementations, missing_debug_implementations, missing_docs)]
#![allow(clippy::new_without_default)]

pub mod event;
pub mod query;
pub mod thirdparty;
