#![doc(html_favicon_url = "https://www.ruma.io/favicon.ico")]
#![doc(html_logo_url = "https://www.ruma.io/images/logo.png")]
//! Common types for other ruma crates.

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod authentication;
pub mod directory;
pub mod encryption;
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
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrivOwnedStr(Box<str>);

impl fmt::Debug for PrivOwnedStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
