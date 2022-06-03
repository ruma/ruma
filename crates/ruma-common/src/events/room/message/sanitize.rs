/// Sanitize the given HTML string.
///
/// This removes the [tags and attributes] that are not listed in the Matrix specification.
///
/// It can also optionally remove the [rich reply fallback].
///
/// [tags and attributes]: https://spec.matrix.org/v1.2/client-server-api/#mroommessage-msgtypes
/// [rich reply fallback]: https://spec.matrix.org/v1.2/client-server-api/#fallbacks-for-rich-replies
#[cfg(feature = "sanitize")]
pub fn sanitize_html(s: &str, remove_reply_fallback: RemoveReplyFallback) -> String {
    let allowed_tags = [
        "font",
        "del",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "blockquote",
        "p",
        "a",
        "ul",
        "ol",
        "sup",
        "sub",
        "li",
        "b",
        "i",
        "u",
        "strong",
        "em",
        "strike",
        "code",
        "hr",
        "br",
        "div",
        "table",
        "thead",
        "tbody",
        "tr",
        "th",
        "td",
        "caption",
        "pre",
        "span",
        "img",
        "details",
        "summary",
    ];

    let allowed_attributes = [
        ("font", vec!["data-mx-bg-color", "data-mx-color", "color"]),
        ("span", vec!["data-mx-bg-color", "data-mx-color", "data-mx-spoiler"]),
        ("a", vec!["name", "target", "href"]),
        ("img", vec!["width", "height", "alt", "title", "src"]),
        ("ol", vec!["start"]),
        ("code", vec!["class"]),
    ];

    let allowed_urls = ["http", "https", "ftp", "mailto", "magnet"];

    let mut builder = ammonia::Builder::empty();
    builder.add_url_schemes(allowed_urls).add_tags(allowed_tags).link_rel(Some("noopener"));

    for (tag, attr) in allowed_attributes {
        builder.add_tag_attributes(tag, attr);
    }

    if remove_reply_fallback == RemoveReplyFallback::Yes {
        builder.add_clean_content_tags(["mx-reply"]);
    } else {
        builder.add_tags(["mx-reply"]);
    }

    builder.clean(s).to_string()
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
    use super::{sanitize_html, RemoveReplyFallback};

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
            RemoveReplyFallback::No
        );

        assert_eq!(
            sanitized,
            "\
            <mx-reply>\
                <blockquote>\
                    <a href=\"https://matrix.to/#/!n8f893n9:example.com/$1598361704261elfgc:localhost\" rel=\"noopener\">In reply to</a> \
                    <a href=\"https://matrix.to/#/@alice:example.com\" rel=\"noopener\">@alice:example.com</a>\
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
