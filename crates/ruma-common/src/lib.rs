#![doc(html_favicon_url = "https://www.ruma.io/favicon.ico")]
#![doc(html_logo_url = "https://www.ruma.io/images/logo.png")]
//! Common types for other ruma crates.

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod authentication;
pub mod directory;
pub mod encryption;
pub mod power_levels;
pub mod presence;
pub mod push;
pub mod receipt;
pub mod thirdparty;
mod time;
pub mod to_device;

pub use time::{MilliSecondsSinceUnixEpoch, SecondsSinceUnixEpoch};
