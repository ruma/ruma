//! Authenticated endpoints for the [content repository].
//!
//! [content repository]: https://spec.matrix.org/latest/client-server-api/#content-repository

#[cfg(feature = "unstable-msc3911")]
pub mod copy_content;
#[cfg(feature = "unstable-msc3911")]
pub mod create_content;
pub mod get_content;
pub mod get_content_as_filename;
pub mod get_content_thumbnail;
pub mod get_media_config;
pub mod get_media_preview;
