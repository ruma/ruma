#![doc(html_favicon_url = "https://ruma.dev/favicon.ico")]
#![doc(html_logo_url = "https://ruma.dev/images/logo.png")]
//! (De)serializable types for the [Matrix Server-Server API][federation-api].
//! These types are used by server code.
//!
//! [federation-api]: https://spec.matrix.org/latest/server-server-api/

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use std::fmt;

mod serde;

pub mod authorization;
pub mod backfill;
pub mod device;
pub mod directory;
pub mod discovery;
pub mod event;
pub mod keys;
pub mod knock;
pub mod membership;
pub mod openid;
pub mod query;
pub mod room;
pub mod space;
pub mod thirdparty;
pub mod transactions;

// Wrapper around `Box<str>` that cannot be used in a meaningful way outside of
// this crate. Used for string enums because their `_Custom` variant can't be
// truly private (only `#[doc(hidden)]`).
#[doc(hidden)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrivOwnedStr(Box<str>);

impl fmt::Debug for PrivOwnedStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
