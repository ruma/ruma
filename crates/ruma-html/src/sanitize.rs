//! Convenience methods and types to sanitize HTML messages.

mod html_sanitizer;

pub use self::html_sanitizer::HtmlSanitizer;

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
    let sanitizer = HtmlSanitizer::new(mode, remove_reply_fallback);
    sanitizer.clean(s).to_string()
}

/// What HTML [tags and attributes] should be kept by the sanitizer.
///
/// [tags and attributes]: https://spec.matrix.org/latest/client-server-api/#mroommessage-msgtypes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::exhaustive_enums)]
pub enum HtmlSanitizerMode {
    /// Keep only the tags and attributes listed in the Matrix specification.
    Strict,

    /// Like `Strict` mode, with additional tags and attributes that are not yet included in
    /// the spec, but are reasonable to keep.
    Compat,
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
    let sanitizer = HtmlSanitizer::reply_fallback_remover();
    sanitizer.clean(s).to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        remove_html_reply_fallback, sanitize_html, HtmlSanitizerMode, RemoveReplyFallback,
    };

    #[test]
    fn sanitize() {
        let sanitized = sanitize_html(
            "\
            <mx-reply>\
                <blockquote>\
                    <a href=\"https://matrix.to/#/!n8f893n9:example.com/$1598361704261elfgc:localhost\">In reply to</a> \
                    <a href=\"https://matrix.to/#/@alice:example.com\">@alice:example.com</a>\
                    <br>\
                    Previous message\
                </blockquote>\
            </mx-reply>\
            <removed>This has no tag</removed>\
            <p>But this is inside a tag</p>\
            ",
            HtmlSanitizerMode::Strict,
            RemoveReplyFallback::No,
        );

        assert_eq!(
            sanitized,
            "\
            <mx-reply>\
                <blockquote>\
                    <a href=\"https://matrix.to/#/!n8f893n9:example.com/$1598361704261elfgc:localhost\">In reply to</a> \
                    <a href=\"https://matrix.to/#/@alice:example.com\">@alice:example.com</a>\
                    <br>\
                    Previous message\
                </blockquote>\
            </mx-reply>\
            This has no tag\
            <p>But this is inside a tag</p>\
            "
        );
    }

    #[test]
    fn sanitize_without_reply() {
        let sanitized = sanitize_html(
            "\
            <mx-reply>\
                <blockquote>\
                    <a href=\"https://matrix.to/#/!n8f893n9:example.com/$1598361704261elfgc:localhost\">In reply to</a> \
                    <a href=\"https://matrix.to/#/@alice:example.com\">@alice:example.com</a>\
                    <br>\
                    Previous message\
                </blockquote>\
            </mx-reply>\
            <removed>This has no tag</removed>\
            <p>But this is inside a tag</p>\
            ",
            HtmlSanitizerMode::Strict,
            RemoveReplyFallback::Yes,
        );

        assert_eq!(
            sanitized,
            "\
            This has no tag\
            <p>But this is inside a tag</p>\
            "
        );
    }

    #[test]
    fn remove_html_reply() {
        let without_reply = remove_html_reply_fallback(
            "\
            <mx-reply>\
                <blockquote>\
                    <a href=\"https://matrix.to/#/!n8f893n9:example.com/$1598361704261elfgc:localhost\">In reply to</a> \
                    <a href=\"https://matrix.to/#/@alice:example.com\">@alice:example.com</a>\
                    <br>\
                    Previous message\
                </blockquote>\
            </mx-reply>\
            <keep-me>This keeps its tag</keep-me>\
            <p>But this is inside a tag</p>\
            ",
        );

        assert_eq!(
            without_reply,
            "\
            <keep-me>This keeps its tag</keep-me>\
            <p>But this is inside a tag</p>\
            "
        );
    }
}
