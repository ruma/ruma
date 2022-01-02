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
pub mod unversioned;

pub use error::Error;
