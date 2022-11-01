#![doc(html_favicon_url = "https://www.ruma.io/favicon.ico")]
#![doc(html_logo_url = "https://www.ruma.io/images/logo.png")]
//! (De)serializable types for the [Matrix Client-Server API][client-api].
//! These types can be shared by client and server code.
//!
//! [client-api]: https://spec.matrix.org/v1.4/client-server-api/

#![cfg(any(feature = "client", feature = "server"))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![warn(missing_docs)] FIXME

pub mod account;
pub mod alias;
pub mod appservice;
pub mod backup;
pub mod config;
pub mod context;
pub mod device;
pub mod directory;
pub mod discovery;
pub mod error;
pub mod filter;
pub mod http_headers;
pub mod keys;
pub mod knock;
pub mod media;
pub mod membership;
pub mod message;
pub mod presence;
pub mod profile;
pub mod push;
pub mod read_marker;
pub mod receipt;
pub mod redact;
pub mod relations;
pub mod room;
pub mod search;
pub mod server;
pub mod session;
pub mod space;
pub mod state;
pub mod sync;
pub mod tag;
pub mod thirdparty;
pub mod threads;
pub mod to_device;
pub mod typing;
pub mod uiaa;
pub mod user_directory;
pub mod voip;

use std::fmt;

pub use error::Error;
use serde::{Deserialize, Serialize};

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

/// The direction to return events from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[allow(clippy::exhaustive_enums)]
pub enum Direction {
    /// Return events backwards in time from the requested `from` token.
    #[default]
    #[serde(rename = "b")]
    Backward,

    /// Return events forwards in time from the requested `from` token.
    #[serde(rename = "f")]
    Forward,
}
