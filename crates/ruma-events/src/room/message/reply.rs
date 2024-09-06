use std::fmt::{self, Write};

use ruma_common::{EventId, RoomId, UserId};
#[cfg(feature = "html")]
use ruma_html::Html;

use super::{
    sanitize::remove_plain_reply_fallback, FormattedBody, MessageType, OriginalRoomMessageEvent,
    Relation,
};

pub(super) struct OriginalEventData<'a> {
    pub(super) body: &'a str,
    pub(super) formatted: Option<&'a FormattedBody>,
    pub(super) is_emote: bool,
    pub(super) is_reply: bool,
    pub(super) room_id: &'a RoomId,
    pub(super) event_id: &'a EventId,
    pub(super) sender: &'a UserId,
}

impl<'a> From<&'a OriginalRoomMessageEvent> for OriginalEventData<'a> {
    fn from(message: &'a OriginalRoomMessageEvent) -> Self {
        let OriginalRoomMessageEvent { room_id, event_id, sender, content, .. } = message;
        let is_reply = matches!(content.relates_to, Some(Relation::Reply { .. }));

        let (body, formatted, is_emote) = match &content.msgtype {
            MessageType::Audio(_) => ("sent an audio file.", None, false),
            MessageType::Emote(c) => (&*c.body, c.formatted.as_ref(), true),
            MessageType::File(_) => ("sent a file.", None, false),
            MessageType::Image(_) => ("sent an image.", None, false),
            MessageType::Location(_) => ("sent a location.", None, false),
            MessageType::Notice(c) => (&*c.body, c.formatted.as_ref(), false),
            MessageType::ServerNotice(c) => (&*c.body, None, false),
            MessageType::Text(c) => (&*c.body, c.formatted.as_ref(), false),
            MessageType::Video(_) => ("sent a video.", None, false),
            MessageType::VerificationRequest(c) => (&*c.body, None, false),
            MessageType::_Custom(c) => (&*c.body, None, false),
        };

        Self { body, formatted, is_emote, is_reply, room_id, event_id, sender }
    }
}

fn get_message_quote_fallbacks(original_event: OriginalEventData<'_>) -> (String, String) {
    let OriginalEventData { body, formatted, is_emote, is_reply, room_id, event_id, sender } =
        original_event;
    let emote_sign = is_emote.then_some("* ").unwrap_or_default();
    let body = is_reply.then(|| remove_plain_reply_fallback(body)).unwrap_or(body);
    #[cfg(feature = "html")]
    let html_body = FormattedOrPlainBody { formatted, body, is_reply };
    #[cfg(not(feature = "html"))]
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
    #[cfg(feature = "html")]
    is_reply: bool,
}

impl fmt::Display for FormattedOrPlainBody<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(formatted_body) = self.formatted {
            #[cfg(feature = "html")]
            if self.is_reply {
                let html = Html::parse(&formatted_body.body);
                html.sanitize();

                write!(f, "{html}")
            } else {
                f.write_str(&formatted_body.body)
            }

            #[cfg(not(feature = "html"))]
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
pub(super) fn plain_and_formatted_reply_body(
    body: &str,
    formatted: Option<impl fmt::Display>,
    original_event: OriginalEventData<'_>,
) -> (String, String) {
    let (quoted, quoted_html) = get_message_quote_fallbacks(original_event);

    let plain = format!("{quoted}\n\n{body}");
    let html = match formatted {
        Some(formatted) => format!("{quoted_html}{formatted}"),
        None => format!("{quoted_html}{}", EscapeHtmlEntities(body)),
    };

    (plain, html)
}

#[cfg(test)]
mod tests {
    use ruma_common::{owned_event_id, owned_room_id, owned_user_id, MilliSecondsSinceUnixEpoch};

    use super::OriginalRoomMessageEvent;
    use crate::{room::message::RoomMessageEventContent, MessageLikeUnsigned};

    #[test]
    fn fallback_multiline() {
        let (plain_quote, html_quote) = super::get_message_quote_fallbacks(
            (&OriginalRoomMessageEvent {
                content: RoomMessageEventContent::text_plain("multi\nline"),
                event_id: owned_event_id!("$1598361704261elfgc:localhost"),
                sender: owned_user_id!("@alice:example.com"),
                origin_server_ts: MilliSecondsSinceUnixEpoch::now(),
                room_id: owned_room_id!("!n8f893n9:example.com"),
                unsigned: MessageLikeUnsigned::new(),
            })
                .into(),
        );

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
