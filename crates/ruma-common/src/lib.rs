#![doc(html_favicon_url = "https://www.ruma.io/favicon.ico")]
#![doc(html_logo_url = "https://www.ruma.io/images/logo.png")]
//! Common types for the Ruma crates.

#![recursion_limit = "1024"]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(not(all(feature = "client", feature = "server")))]
compile_error!(
    "ruma_common's `client` and `server` Cargo features only exist as a workaround are not meant to be disabled"
);

// Hack to allow both ruma-common itself and external crates (or tests) to use procedural macros
// that expect `ruma_common` to exist in the prelude.
extern crate self as ruma_common;

#[cfg(feature = "account-data")]
pub mod account_data;
#[cfg(feature = "api")]
pub mod api;
pub mod authentication;
pub mod directory;
pub mod encryption;
#[cfg(feature = "events")]
pub mod events;
pub mod power_levels;
pub mod presence;
pub mod push;
pub mod receipt;
pub mod room;
pub mod thirdparty;
mod time;
pub mod to_device;

use std::fmt;

pub use time::{MilliSecondsSinceUnixEpoch, SecondsSinceUnixEpoch};

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

/// Re-exports used by macro-generated code.
///
/// It is not considered part of this module's public API.
#[doc(hidden)]
pub mod exports {
    pub use ruma_identifiers;
    pub use ruma_serde;
    pub use serde;
    pub use serde_json;
}
