//! Endpoints for the media repository.

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
