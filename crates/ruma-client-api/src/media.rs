//! Endpoints for the media repository.

use std::time::Duration;

pub mod create_content;
pub mod create_content_async;
pub mod create_mxc_uri;
pub mod get_content;
pub mod get_content_as_filename;
pub mod get_content_thumbnail;
pub mod get_media_config;
pub mod get_media_preview;

/// The default duration that the client should be willing to wait to start receiving data.
pub(crate) fn default_download_timeout() -> Duration {
    Duration::from_secs(20)
}

/// Whether the given duration is the default duration that the client should be willing to wait to
/// start receiving data.
pub(crate) fn is_default_download_timeout(timeout: &Duration) -> bool {
    timeout.as_secs() == 20
}
