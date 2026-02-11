#![doc(html_favicon_url = "https://ruma.dev/favicon.ico")]
#![doc(html_logo_url = "https://ruma.dev/images/logo.png")]
//! (De)serializable types for the [Matrix Identity Service API][identity-api].
//! These types can be shared by client and identity service code.
//!
//! [identity-api]: https://spec.matrix.org/latest/identity-service-api/

#![warn(missing_docs)]

pub mod association;
pub mod authentication;
pub mod discovery;
pub mod invitation;
pub mod keys;
pub mod lookup;
pub mod tos;

ruma_common::priv_owned_str!();
