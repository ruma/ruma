//! Custom HTTP headers not defined in the `http` crate.
#![allow(clippy::declare_interior_mutable_const)]

use http::header::HeaderName;

/// The [`Cross-Origin-Resource-Policy`] HTTP response header.
///
/// [`Cross-Origin-Resource-Policy`]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cross-Origin-Resource-Policy
pub const CROSS_ORIGIN_RESOURCE_POLICY: HeaderName =
    HeaderName::from_static("cross-origin-resource-policy");
