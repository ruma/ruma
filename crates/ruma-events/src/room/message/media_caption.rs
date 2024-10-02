//! Reusable methods for captioning media files.

use crate::room::message::FormattedBody;

/// Computes the caption of a media file as defined by the [spec](https://spec.matrix.org/latest/client-server-api/#media-captions).
///
/// In short, this is the `body` field if the `filename` field exists and has a different value,
/// otherwise the media file does not have a caption.
pub(crate) fn caption<'a>(body: &'a str, filename: Option<&str>) -> Option<&'a str> {
    filename.is_some_and(|filename| body != filename).then_some(body)
}

/// Computes the formatted caption of a media file as defined by the [spec](https://spec.matrix.org/latest/client-server-api/#media-captions).
///
/// This is the same as `caption`, but returns the formatted body instead of the plain body.
pub(crate) fn formatted_caption<'a>(
    body: &str,
    formatted: Option<&'a FormattedBody>,
    filename: Option<&str>,
) -> Option<&'a FormattedBody> {
    filename.is_some_and(|filename| body != filename).then_some(formatted).flatten()
}
