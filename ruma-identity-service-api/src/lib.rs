#![doc(html_favicon_url = "https://www.ruma.io/favicon.ico")]
#![doc(html_logo_url = "https://www.ruma.io/images/logo.png")]
//! (De)serializable types for the [Matrix Identity Service API][identity-api].
//! These types can be shared by client and identity service code.
//!
//! [identity-api]: https://matrix.org/docs/spec/identity_service/r0.3.0.html

#![warn(missing_docs)]

pub mod association;
pub mod authentication;
pub mod keys;
pub mod lookup;
pub mod status;
pub mod tos;
