#![doc(html_favicon_url = "https://ruma.dev/favicon.ico")]
#![doc(html_logo_url = "https://ruma.dev/images/logo.png")]
//! Opinionated HTML parsing and manipulating library.
//!
//! Like the rest of the Ruma crates, this crate is primarily meant to be used for
//! the Matrix protocol. It should be able to be used to interact with any HTML
//! document but will offer APIs focused on specificities of HTML in the Matrix
//! specification..
//!
//! # Features
//!
//! * `matrix` - Allow to convert HTML elements data into enums with variants for elements and
//!   attributes [suggested by the Matrix Specification][spec].
//!
//! [spec]: https://spec.matrix.org/latest/client-server-api/#mroommessage-msgtypes

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub use html5ever::{tendril::StrTendril, Attribute, LocalName, Namespace, Prefix, QualName};

mod helpers;
mod html;
mod sanitizer_config;

pub use self::{helpers::*, html::*, sanitizer_config::*};

/// What [HTML elements and attributes] should be kept by the sanitizer.
///
/// [HTML elements and attributes]: https://spec.matrix.org/latest/client-server-api/#mroommessage-msgtypes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::exhaustive_enums)]
pub enum HtmlSanitizerMode {
    /// Keep only the elements and attributes suggested in the Matrix specification.
    ///
    /// In addition to filtering elements and attributes listed in the Matrix specification, it
    /// also removes elements that are nested more than 100 levels deep.
    ///
    /// Deprecated elements and attributes are also replaced when applicable.
    Strict,

    /// Like `Strict` mode, with additional elements and attributes that are not yet included in
    /// the spec, but are reasonable to keep.
    ///
    /// Differences with `Strict` mode:
    ///
    /// * The `matrix` scheme is allowed in links.
    Compat,
}
