//! Endpoints and helpers for the media repository.

pub mod create_content;
pub mod create_content_async;
pub mod create_mxc_uri;
pub mod get_content;
pub mod get_content_as_filename;
pub mod get_content_thumbnail;
pub mod get_media_config;
pub mod get_media_preview;

/// Checks whether a given `Content-Type` is considered "safe" for having a `Content-Disposition` of
/// `inline` returned on `/download`, as recommended by the [spec].
///
/// [spec]: https://spec.matrix.org/v1.12/client-server-api/#serving-inline-content
pub fn is_safe_inline_content_type(content_type: &str) -> bool {
    const SAFE_CONTENT_TYPES: [&str; 26] = [
        "text/css",
        "text/plain",
        "text/csv",
        "application/json",
        "application/ld+json",
        "image/jpeg",
        "image/gif",
        "image/png",
        "image/apng",
        "image/webp",
        "image/avif",
        "video/mp4",
        "video/webm",
        "video/ogg",
        "video/quicktime",
        "audio/mp4",
        "audio/webm",
        "audio/aac",
        "audio/mpeg",
        "audio/ogg",
        "audio/wave",
        "audio/wav",
        "audio/x-wav",
        "audio/x-pn-wav",
        "audio/flac",
        "audio/x-flac",
    ];

    SAFE_CONTENT_TYPES.contains(&content_type)
}
