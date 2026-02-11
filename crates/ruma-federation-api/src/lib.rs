#![doc(html_favicon_url = "https://ruma.dev/favicon.ico")]
#![doc(html_logo_url = "https://ruma.dev/images/logo.png")]
//! (De)serializable types for the [Matrix Server-Server API][federation-api].
//! These types are used by server code.
//!
//! [federation-api]: https://spec.matrix.org/latest/server-server-api/

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod serde;

pub mod authenticated_media;
pub mod authentication;
pub mod authorization;
pub mod backfill;
pub mod device;
pub mod directory;
pub mod discovery;
pub mod event;
pub mod keys;
pub mod membership;
pub mod openid;
pub mod query;
pub mod room;
pub mod space;
pub mod thirdparty;
pub mod transactions;

ruma_common::priv_owned_str!();
