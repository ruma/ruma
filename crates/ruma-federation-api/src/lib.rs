#![doc(html_favicon_url = "https://www.ruma.io/favicon.ico")]
#![doc(html_logo_url = "https://www.ruma.io/images/logo.png")]
//! (De)serializable types for the [Matrix Server-Server API][federation-api].
//! These types are used by server code.
//!
//! [federation-api]: https://matrix.org/docs/spec/server_server/r0.1.4.html

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod serde;

pub mod authorization;
pub mod backfill;
pub mod device;
pub mod directory;
pub mod discovery;
pub mod event;
pub mod keys;
#[cfg(feature = "unstable-pre-spec")]
pub mod knock;
pub mod membership;
pub mod openid;
pub mod query;
pub mod thirdparty;
pub mod transactions;
