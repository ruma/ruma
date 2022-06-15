//! Convenience methods and types to sanitize text messages.

#[cfg(feature = "sanitize")]
mod html_sanitizer;

#[cfg(feature = "sanitize")]
use html_sanitizer::HtmlSanitizer;

/// Sanitize the given HTML string.
///
/// This removes the [tags and attributes] that are not listed in the Matrix specification.
///
/// It can also optionally remove the [rich reply fallback].
///
/// [tags and attributes]: https://spec.matrix.org/v1.2/client-server-api/#mroommessage-msgtypes
/// [rich reply fallback]: https://spec.matrix.org/v1.2/client-server-api/#fallbacks-for-rich-replies
#[cfg(feature = "sanitize")]
pub fn sanitize_html(
    s: &str,
    mode: HtmlSanitizerMode,
    remove_reply_fallback: RemoveReplyFallback,
) -> String {
    let sanitizer = HtmlSanitizer::new(mode, remove_reply_fallback);
    sanitizer.clean(s)
}

/// What HTML [tags and attributes] should be kept by the sanitizer.
///
/// [tags and attributes]: https://spec.matrix.org/v1.2/client-server-api/#mroommessage-msgtypes
#[cfg(feature = "sanitize")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::exhaustive_enums)]
pub enum HtmlSanitizerMode {
    /// Keep only the tags and attributes listed in the Matrix specification.
    Strict,

    /// Keeps all the tags and attributes in `Strict` mode, and others that are not in this section
    /// of the spec, but might be encountered.
    Compat,
}

/// Whether to remove the [rich reply fallback] while sanitizing.
///
/// [rich reply fallback]: https://spec.matrix.org/v1.2/client-server-api/#fallbacks-for-rich-replies
#[cfg(feature = "sanitize")]
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
/// [rich reply fallback]: https://spec.matrix.org/v1.2/client-server-api/#fallbacks-for-rich-replies
#[cfg(feature = "sanitize")]
pub fn remove_html_reply_fallback(s: &str) -> String {
    let sanitizer = HtmlSanitizer::reply_fallback_remover();
    sanitizer.clean(s)
}

/// Remove the [rich reply fallback] of the given plain text string.
///
/// [rich reply fallback]: https://spec.matrix.org/v1.2/client-server-api/#fallbacks-for-rich-replies
pub fn remove_plain_reply_fallback(s: &str) -> &str {
    if !s.starts_with("> ") {
        s
    } else {
        let mut start = 0;
        for (pos, _) in s.match_indices('\n') {
            if !&s[pos + 1..].starts_with("> ") {
                start = pos + 1;
                break;
            }
        }
        &s[start..]
    }
}

#[cfg(test)]
mod tests {
    use super::remove_plain_reply_fallback;
    #[cfg(feature = "sanitize")]
    use super::{
        remove_html_reply_fallback, sanitize_html, HtmlSanitizerMode, RemoveReplyFallback,
    };

    #[test]
    #[cfg(feature = "sanitize")]
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
    #[cfg(feature = "sanitize")]
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
            RemoveReplyFallback::Yes
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
    #[cfg(feature = "sanitize")]
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

    #[test]
    fn remove_plain_reply() {
        assert_eq!(
            remove_plain_reply_fallback("No reply here\nJust a simple message"),
            "No reply here\nJust a simple message"
        );

        assert_eq!(
            remove_plain_reply_fallback(
                "> <@user:notareal.hs> Replied to on\n> two lines\nThis is my reply"
            ),
            "This is my reply"
        );

        assert_eq!(remove_plain_reply_fallback("\n> Not on first line"), "\n> Not on first line");

        assert_eq!(
            remove_plain_reply_fallback("> <@user:notareal.hs> Previous message\n\n> New quote"),
            "\n> New quote"
        );
    }
}
