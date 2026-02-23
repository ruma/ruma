#![doc(html_favicon_url = "https://ruma.dev/favicon.ico")]
#![doc(html_logo_url = "https://ruma.dev/images/logo.png")]
//! Common types for the Ruma crates.

#![recursion_limit = "1024"]
#![warn(missing_docs)]
// https://github.com/rust-lang/rust-clippy/issues/9029
#![allow(clippy::derive_partial_eq_without_eq)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(not(all(feature = "client", feature = "server")))]
compile_error!(
    "ruma_common's `client` and `server` Cargo features only exist as a workaround are not meant to be disabled"
);

// Hack to allow both ruma-common itself and external crates (or tests) to use procedural macros
// that expect `ruma_common` to exist in the prelude.
extern crate self as ruma_common;

#[cfg(feature = "api")]
pub mod api;
pub mod authentication;
pub mod canonical_json;
pub mod directory;
pub mod encryption;
#[cfg(feature = "api")]
pub mod http_headers;
mod identifiers;
pub mod media;
mod percent_encode;
pub mod power_levels;
pub mod presence;
pub mod push;
pub mod room;
pub mod room_version_rules;
pub mod serde;
pub mod third_party_invite;
pub mod thirdparty;
mod time;
pub mod to_device;

use std::fmt;

pub use self::{
    canonical_json::{CanonicalJsonError, CanonicalJsonObject, CanonicalJsonValue},
    identifiers::*,
    time::{MilliSecondsSinceUnixEpoch, SecondsSinceUnixEpoch},
};

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
    pub use arc_interner;
    pub use arcstr;
    #[cfg(feature = "api")]
    pub use bytes;
    pub use compact_str;
    #[cfg(feature = "api")]
    pub use http;
    pub use ruma_macros;
    pub use serde;
    pub use serde_html_form;
    pub use serde_json;
    pub use smallvec;
}
