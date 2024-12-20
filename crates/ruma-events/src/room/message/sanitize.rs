//! Convenience methods and types to sanitize text messages.

/// Remove the [rich reply] fallback of the given plain text string.
///
/// [rich reply]: https://spec.matrix.org/latest/client-server-api/#rich-replies
pub fn remove_plain_reply_fallback(mut s: &str) -> &str {
    // A reply fallback must begin with a mention of the original sender between `<` and `>`, and
    // emotes add `*` as a prefix. If there is no newline, removing the detected fallback would
    // result in an empty string.
    if (!s.starts_with("> <") && !s.starts_with("> * <")) || !s.contains('\n') {
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
