#![doc(html_favicon_url = "https://www.ruma.io/favicon.ico")]
#![doc(html_logo_url = "https://www.ruma.io/images/logo.png")]
//! (De)serializable types for the [Matrix Push Gateway API][push-api].
//! These types can be shared by push gateway and server code.
//!
//! [push-api]: https://matrix.org/docs/spec/push_gateway/r0.1.1.html

// #![feature(type_alias_impl_trait)]
#![feature(generic_associated_types)]
#![warn(missing_docs)]

pub mod send_event_notification;

// Wrapper around `Box<str>` that cannot be used in a meaningful way outside of
// this crate. Used for string enums because their `_Custom` variant can't be
// truly private (only `#[doc(hidden)]`).
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrivOwnedStr(Box<str>);
