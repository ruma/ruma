#![doc(html_favicon_url = "https://ruma.dev/favicon.ico")]
#![doc(html_logo_url = "https://ruma.dev/images/logo.png")]
//! Opinionated HTML parsing and manipulating library.
//!
//! Like the rest of the Ruma crates, this crate is primarily meant to be used for
//! the Matrix protocol. It should be able to be used to interact with any HTML
//! document but will offer APIs focused on specificities of HTML in the Matrix
//! specification..

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod helpers;
mod html;
mod sanitizer_config;

pub use self::{helpers::*, html::*, sanitizer_config::SanitizerConfig};
