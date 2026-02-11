#![doc(html_favicon_url = "https://ruma.dev/favicon.ico")]
#![doc(html_logo_url = "https://ruma.dev/images/logo.png")]
//! (De)serializable types for the [Matrix Client-Server API][client-api].
//! These types can be shared by client and server code.
//!
//! [client-api]: https://spec.matrix.org/latest/client-server-api/

#![cfg(any(feature = "client", feature = "server"))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

pub mod account;
pub mod alias;
pub mod appservice;
pub mod authenticated_media;
pub mod backup;
pub mod config;
pub mod context;
#[cfg(feature = "unstable-msc3814")]
pub mod dehydrated_device;
#[cfg(feature = "unstable-msc4140")]
pub mod delayed_events;
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
pub mod peeking;
pub mod presence;
pub mod profile;
pub mod push;
pub mod read_marker;
pub mod receipt;
pub mod redact;
pub mod relations;
#[cfg(any(feature = "unstable-msc4108", feature = "unstable-msc4388"))]
pub mod rendezvous;
pub mod reporting;
pub mod room;
#[cfg(feature = "unstable-msc4143")]
pub mod rtc;
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

pub use error::Error;

ruma_common::priv_owned_str!();
