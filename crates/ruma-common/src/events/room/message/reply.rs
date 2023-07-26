use std::fmt::{self, Write};

use super::{
    sanitize::remove_plain_reply_fallback, FormattedBody, MessageType, OriginalRoomMessageEvent,
    Relation,
};
#[cfg(feature = "unstable-sanitize")]
use super::{sanitize::HtmlSanitizer, HtmlSanitizerMode, RemoveReplyFallback};

fn get_message_quote_fallbacks(original_message: &OriginalRoomMessageEvent) -> (String, String) {
    let get_quotes = |body: &str, formatted: Option<&FormattedBody>, is_emote: bool| {
        let OriginalRoomMessageEvent { room_id, event_id, sender, content, .. } = original_message;
        let is_reply = matches!(content.relates_to, Some(Relation::Reply { .. }));
        let emote_sign = is_emote.then_some("* ").unwrap_or_default();
        let body = is_reply.then(|| remove_plain_reply_fallback(body)).unwrap_or(body);
        #[cfg(feature = "unstable-sanitize")]
        let html_body = FormattedOrPlainBody { formatted, body, is_reply };
        #[cfg(not(feature = "unstable-sanitize"))]
        let html_body = FormattedOrPlainBody { formatted, body };

        (
            format!("> {emote_sign}<{sender}> {body}").replace('\n', "\n> "),
            format!(
                "<mx-reply>\
                        <blockquote>\
                            <a href=\"https://matrix.to/#/{room_id}/{event_id}\">In reply to</a> \
                            {emote_sign}<a href=\"https://matrix.to/#/{sender}\">{sender}</a>\
                            <br>\
                            {html_body}\
                        </blockquote>\
                    </mx-reply>"
            ),
        )
    };

    match &original_message.content.msgtype {
        MessageType::Audio(_) => get_quotes("sent an audio file.", None, false),
        MessageType::Emote(c) => get_quotes(&c.body, c.formatted.as_ref(), true),
        MessageType::File(_) => get_quotes("sent a file.", None, false),
        MessageType::Image(_) => get_quotes("sent an image.", None, false),
        MessageType::Location(_) => get_quotes("sent a location.", None, false),
        MessageType::Notice(c) => get_quotes(&c.body, c.formatted.as_ref(), false),
        MessageType::ServerNotice(c) => get_quotes(&c.body, None, false),
        MessageType::Text(c) => get_quotes(&c.body, c.formatted.as_ref(), false),
        MessageType::Video(_) => get_quotes("sent a video.", None, false),
        MessageType::VerificationRequest(content) => get_quotes(&content.body, None, false),
        MessageType::_Custom(content) => get_quotes(&content.body, None, false),
    }
}

struct EscapeHtmlEntities<'a>(&'a str);

impl fmt::Display for EscapeHtmlEntities<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in self.0.chars() {
            // Escape reserved HTML entities and new lines.
            // <https://developer.mozilla.org/en-US/docs/Glossary/Entity#reserved_characters>
            match c {
                '&' => f.write_str("&amp;")?,
                '<' => f.write_str("&lt;")?,
                '>' => f.write_str("&gt;")?,
                '"' => f.write_str("&quot;")?,
                '\n' => f.write_str("<br>")?,
                _ => f.write_char(c)?,
            }
        }

        Ok(())
    }
}

struct FormattedOrPlainBody<'a> {
    formatted: Option<&'a FormattedBody>,
    body: &'a str,
    #[cfg(feature = "unstable-sanitize")]
    is_reply: bool,
}

impl fmt::Display for FormattedOrPlainBody<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(formatted_body) = self.formatted {
            #[cfg(feature = "unstable-sanitize")]
            if self.is_reply {
                let sanitizer =
                    HtmlSanitizer::new(HtmlSanitizerMode::Strict, RemoveReplyFallback::Yes);
                write!(f, "{}", sanitizer.clean(&formatted_body.body))
            } else {
                f.write_str(&formatted_body.body)
            }

            #[cfg(not(feature = "unstable-sanitize"))]
            f.write_str(&formatted_body.body)
        } else {
            write!(f, "{}", EscapeHtmlEntities(self.body))
        }
    }
}

/// Get the plain and formatted body for a rich reply.
///
/// Returns a `(plain, html)` tuple.
///
/// With the `sanitize` feature, [HTML tags and attributes] that are not allowed in the Matrix
/// spec and previous [rich reply fallbacks] are removed from the previous message in the new rich
/// reply fallback.
///
/// [HTML tags and attributes]: https://spec.matrix.org/latest/client-server-api/#mroommessage-msgtypes
/// [rich reply fallbacks]: https://spec.matrix.org/latest/client-server-api/#fallbacks-for-rich-replies
pub(crate) fn plain_and_formatted_reply_body(
    body: &str,
    formatted: Option<impl fmt::Display>,
    original_message: &OriginalRoomMessageEvent,
) -> (String, String) {
    let (quoted, quoted_html) = get_message_quote_fallbacks(original_message);

    let plain = format!("{quoted}\n\n{body}");
    let html = match formatted {
        Some(formatted) => format!("{quoted_html}{formatted}"),
        None => format!("{quoted_html}{}", EscapeHtmlEntities(body)),
    };

    (plain, html)
}

#[cfg(test)]
mod tests {
    use crate::{
        events::{room::message::RoomMessageEventContent, MessageLikeUnsigned},
        owned_event_id, owned_room_id, owned_user_id, MilliSecondsSinceUnixEpoch,
    };

    use super::OriginalRoomMessageEvent;

    #[test]
    fn fallback_multiline() {
        let (plain_quote, html_quote) =
            super::get_message_quote_fallbacks(&OriginalRoomMessageEvent {
                content: RoomMessageEventContent::text_plain("multi\nline"),
                event_id: owned_event_id!("$1598361704261elfgc:localhost"),
                sender: owned_user_id!("@alice:example.com"),
                origin_server_ts: MilliSecondsSinceUnixEpoch::now(),
                room_id: owned_room_id!("!n8f893n9:example.com"),
                unsigned: MessageLikeUnsigned::new(),
            });

        assert_eq!(plain_quote, "> <@alice:example.com> multi\n> line");
        assert_eq!(
            html_quote,
            "<mx-reply>\
                <blockquote>\
                    <a href=\"https://matrix.to/#/!n8f893n9:example.com/$1598361704261elfgc:localhost\">In reply to</a> \
                    <a href=\"https://matrix.to/#/@alice:example.com\">@alice:example.com</a>\
                    <br>\
                    multi<br>line\
                </blockquote>\
            </mx-reply>",
        );
    }
}
