//! Crate ruma_appservice_api contains serializable types for the requests and responses for each
//! endpoint in the [Matrix](https://matrix.org/) application service API specification. These
//! types can be shared by application service and server code.

#![warn(missing_debug_implementations, missing_docs)]

pub mod event;
pub mod query;
pub mod thirdparty;
