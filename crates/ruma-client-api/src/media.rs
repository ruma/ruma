//! Endpoints for the media repository.

#[cfg(feature = "unstable-msc2246")]
use std::time::Duration;

pub mod create_content;
#[cfg(feature = "unstable-msc2246")]
pub mod create_content_async;
#[cfg(feature = "unstable-msc2246")]
pub mod create_mxc_uri;
pub mod get_content;
pub mod get_content_as_filename;
pub mod get_content_thumbnail;
pub mod get_media_config;
pub mod get_media_preview;

/// The default duration that the client should be willing to wait to start receiving data.
#[cfg(feature = "unstable-msc2246")]
fn default_download_timeout() -> Duration {
    Duration::from_secs(20)
}

/// Whether the given duration is the default duration that the client should be willing to wait to
/// start receiving data.
#[cfg(feature = "unstable-msc2246")]
fn is_default_download_timeout(timeout: &Duration) -> bool {
    timeout.as_secs() == 20
}
