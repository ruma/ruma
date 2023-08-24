//! Convenience methods and types to sanitize text messages.

/// Whether to remove the [rich reply fallback] while sanitizing.
///
/// [rich reply fallback]: https://spec.matrix.org/latest/client-server-api/#fallbacks-for-rich-replies
#[cfg(feature = "html")]
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
/// [rich reply fallback]: https://spec.matrix.org/latest/client-server-api/#fallbacks-for-rich-replies
pub fn remove_plain_reply_fallback(mut s: &str) -> &str {
    if !s.starts_with("> ") {
        return s;
    }

    while s.starts_with("> ") {
        if let Some((_line, rest)) = s.split_once('\n') {
            s = rest;
        } else {
            return "";
        }
    }

    // Strip the first line after the fallback if it is empty.
    if let Some(rest) = s.strip_prefix('\n') {
        rest
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::remove_plain_reply_fallback;

    #[test]
    fn remove_plain_reply() {
        assert_eq!(
            remove_plain_reply_fallback("No reply here\nJust a simple message"),
            "No reply here\nJust a simple message"
        );

        assert_eq!(
            remove_plain_reply_fallback(
                "> <@user:notareal.hs> Replied to on\n\
                 > two lines\n\
                 \n\
                 \n\
                 This is my reply"
            ),
            "\nThis is my reply"
        );

        assert_eq!(remove_plain_reply_fallback("\n> Not on first line"), "\n> Not on first line");

        assert_eq!(
            remove_plain_reply_fallback("> <@user:notareal.hs> Previous message\n\n> New quote"),
            "> New quote"
        );
    }
}
