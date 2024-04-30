//! Convenience methods and types to sanitize HTML messages.

use crate::{Html, HtmlSanitizerMode, SanitizerConfig};

/// Sanitize the given HTML string.
///
/// This removes the [tags and attributes] that are not listed in the Matrix specification.
///
/// It can also optionally remove the [rich reply fallback].
///
/// [tags and attributes]: https://spec.matrix.org/latest/client-server-api/#mroommessage-msgtypes
/// [rich reply fallback]: https://spec.matrix.org/latest/client-server-api/#fallbacks-for-rich-replies
pub fn sanitize_html(
    s: &str,
    mode: HtmlSanitizerMode,
    remove_reply_fallback: RemoveReplyFallback,
) -> String {
    let mut config = match mode {
        HtmlSanitizerMode::Strict => SanitizerConfig::strict(),
        HtmlSanitizerMode::Compat => SanitizerConfig::compat(),
    };

    if remove_reply_fallback == RemoveReplyFallback::Yes {
        config = config.remove_reply_fallback();
    }

    sanitize_inner(s, &config)
}

/// Whether to remove the [rich reply fallback] while sanitizing.
///
/// [rich reply fallback]: https://spec.matrix.org/latest/client-server-api/#fallbacks-for-rich-replies
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::exhaustive_enums)]
pub enum RemoveReplyFallback {
    /// Remove the rich reply fallback.
    Yes,

    /// Don't remove the rich reply fallback.
    No,
}

/// Remove the [rich reply fallback] of the given HTML string.
///
/// Due to the fact that the HTML is parsed, note that malformed HTML and comments will be stripped
/// from the output.
///
/// [rich reply fallback]: https://spec.matrix.org/latest/client-server-api/#fallbacks-for-rich-replies
pub fn remove_html_reply_fallback(s: &str) -> String {
    let config = SanitizerConfig::new().remove_reply_fallback();
    sanitize_inner(s, &config)
}

fn sanitize_inner(s: &str, config: &SanitizerConfig) -> String {
    let mut html = Html::parse(s);
    html.sanitize_with(config);
    html.to_string()
}
