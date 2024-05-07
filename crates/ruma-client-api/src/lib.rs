#![doc(html_favicon_url = "https://ruma.dev/favicon.ico")]
#![doc(html_logo_url = "https://ruma.dev/images/logo.png")]
//! (De)serializable types for the [Matrix Client-Server API][client-api].
//! These types can be shared by client and server code.
//!
//! [client-api]: https://spec.matrix.org/latest/client-server-api/

#![cfg(any(feature = "client", feature = "server"))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(missing_docs)]

pub mod account;
pub mod alias;
pub mod appservice;
pub mod backup;
pub mod config;
pub mod context;
#[cfg(feature = "unstable-msc3814")]
pub mod dehydrated_device;
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
#[cfg(feature = "unstable-msc4108")]
pub mod rendezvous;
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
