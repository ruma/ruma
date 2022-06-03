use std::fmt;

use super::{remove_plain_reply_fallback, FormattedBody, MessageType, OriginalRoomMessageEvent};
#[cfg(feature = "sanitize")]
use super::{sanitize_html, RemoveReplyFallback};

fn get_message_quote_fallbacks(original_message: &OriginalRoomMessageEvent) -> (String, String) {
    match &original_message.content.msgtype {
        MessageType::Audio(_) => get_quotes("sent an audio file.", None, original_message, false),
        MessageType::Emote(content) => {
            get_quotes(&content.body, content.formatted.as_ref(), original_message, true)
        }
        MessageType::File(_) => get_quotes("sent a file.", None, original_message, false),
        MessageType::Image(_) => get_quotes("sent an image.", None, original_message, false),
        MessageType::Location(_) => get_quotes("sent a location.", None, original_message, false),
        MessageType::Notice(content) => {
            get_quotes(&content.body, content.formatted.as_ref(), original_message, false)
        }
        MessageType::ServerNotice(content) => {
            get_quotes(&content.body, None, original_message, false)
        }
        MessageType::Text(content) => {
            get_quotes(&content.body, content.formatted.as_ref(), original_message, false)
        }
        MessageType::Video(_) => get_quotes("sent a video.", None, original_message, false),
        MessageType::_Custom(content) => get_quotes(&content.body, None, original_message, false),
        MessageType::VerificationRequest(content) => {
            get_quotes(&content.body, None, original_message, false)
        }
    }
}

fn get_quotes(
    body: &str,
    formatted: Option<&FormattedBody>,
    original_message: &OriginalRoomMessageEvent,
    is_emote: bool,
) -> (String, String) {
    let OriginalRoomMessageEvent { room_id, event_id, sender, .. } = original_message;
    let emote_sign = is_emote.then(|| "* ").unwrap_or_default();
    let body = remove_plain_reply_fallback(body);
    let html_body = formatted_or_plain_body(formatted, body);

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
}

fn formatted_or_plain_body(formatted: Option<&FormattedBody>, body: &str) -> String {
    if let Some(formatted_body) = formatted {
        #[cfg(feature = "sanitize")]
        {
            sanitize_html(&formatted_body.body, RemoveReplyFallback::Yes)
        }

        #[cfg(not(feature = "sanitize"))]
        formatted_body.body.clone()
    } else {
        let mut escaped_body = String::with_capacity(body.len());
        for c in body.chars() {
            let s = match c {
                '&' => Some("&amp;"),
                '<' => Some("&lt;"),
                '>' => Some("&gt;"),
                '"' => Some("&quot;"),
                '\'' => Some("&apos;"),
                '\n' => Some("<br>"),
                _ => None,
            };
            if let Some(s) = s {
                escaped_body.push_str(s);
            } else {
                escaped_body.push(c);
            }
        }
        escaped_body
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
/// [HTML tags and attributes]: https://spec.matrix.org/v1.2/client-server-api/#mroommessage-msgtypes
/// [rich reply fallbacks]: https://spec.matrix.org/v1.2/client-server-api/#fallbacks-for-rich-replies
pub fn plain_and_formatted_reply_body(
    body: impl fmt::Display,
    formatted: Option<impl fmt::Display>,
    original_message: &OriginalRoomMessageEvent,
) -> (String, String) {
    let (quoted, quoted_html) = get_message_quote_fallbacks(original_message);

    let plain = format!("{quoted}\n{body}");
    let html = if let Some(formatted) = formatted {
        format!("{quoted_html}{formatted}")
    } else {
        format!("{quoted_html}{body}")
    };

    (plain, html)
}

#[cfg(test)]
mod tests {
    use crate::{
        event_id,
        events::{room::message::RoomMessageEventContent, MessageLikeUnsigned},
        room_id, user_id, MilliSecondsSinceUnixEpoch,
    };

    use super::OriginalRoomMessageEvent;

    #[test]
    fn fallback_multiline() {
        let (plain_quote, html_quote) =
            super::get_message_quote_fallbacks(&OriginalRoomMessageEvent {
                content: RoomMessageEventContent::text_plain("multi\nline"),
                event_id: event_id!("$1598361704261elfgc:localhost").to_owned(),
                sender: user_id!("@alice:example.com").to_owned(),
                origin_server_ts: MilliSecondsSinceUnixEpoch::now(),
                room_id: room_id!("!n8f893n9:example.com").to_owned(),
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
            </mx-reply>"
        );
    }
}
