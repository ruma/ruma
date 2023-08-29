#![doc(html_favicon_url = "https://www.ruma.io/favicon.ico")]
#![doc(html_logo_url = "https://www.ruma.io/images/logo.png")]
//! Opinionated HTML parsing and manipulating library.
//!
//! Like the rest of the Ruma crates, this crate is primarily meant to be used for
//! the Matrix protocol. It should be able to be used to interact with any HTML
//! document but will offer APIs focused on specificities of HTML in the Matrix
//! specification..

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod html;
mod sanitize;

pub use self::{
    html::{ElementData, Html, Node},
    sanitize::*,
};
