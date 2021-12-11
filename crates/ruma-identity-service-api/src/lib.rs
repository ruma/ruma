#![doc(html_favicon_url = "https://www.ruma.io/favicon.ico")]
#![doc(html_logo_url = "https://www.ruma.io/images/logo.png")]
//! (De)serializable types for the [Matrix Identity Service API][identity-api].
//! These types can be shared by client and identity service code.
//!
//! [identity-api]: https://matrix.org/docs/spec/identity_service/r0.3.0.html

#![feature(generic_associated_types)]
#![warn(missing_docs)]

pub mod association;
pub mod authentication;
pub mod invitation;
pub mod keys;
pub mod lookup;
pub mod status;
pub mod tos;

// Wrapper around `Box<str>` that cannot be used in a meaningful way outside of
// this crate. Used for string enums because their `_Custom` variant can't be
// truly private (only `#[doc(hidden)]`).
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrivOwnedStr(Box<str>);
