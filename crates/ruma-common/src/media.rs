//! Common types and functions for the [content repository].
//!
//! [content repository]: https://spec.matrix.org/latest/client-server-api/#content-repository

use std::time::Duration;

use crate::{serde::StringEnum, PrivOwnedStr};

/// The desired resizing method for a thumbnail.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Method {
    /// Crop the original to produce the requested image dimensions.
    Crop,

    /// Maintain the original aspect ratio of the source image.
    Scale,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The default duration that the client should be willing to wait to start receiving data.
pub fn default_download_timeout() -> Duration {
    Duration::from_secs(20)
}

/// Whether the given duration is the default duration that the client should be willing to wait to
/// start receiving data.
pub fn is_default_download_timeout(timeout: &Duration) -> bool {
    timeout.as_secs() == 20
}
