#![doc(html_favicon_url = "https://www.ruma.io/favicon.ico")]
#![doc(html_logo_url = "https://www.ruma.io/images/logo.png")]
//! (De)serializable types for the [Matrix Client-Server API][client-api].
//! These types can be shared by client and server code.
//!
//! [client-api]: https://matrix.org/docs/spec/client_server/r0.6.1.html

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod error;
pub mod r0;
pub mod session;
pub mod unversioned;

use std::fmt;

pub use error::Error;

// Wrapper around `Box<str>` that cannot be used in a meaningful way outside of
// this crate. Used for string enums because their `_Custom` variant can't be
// truly private (only `#[doc(hidden)]`).
#[doc(hidden)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrivOwnedStr(Box<str>);

impl fmt::Debug for PrivOwnedStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
