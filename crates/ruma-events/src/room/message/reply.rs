use indoc::formatdoc;

use super::{FormattedBody, MessageType, RoomMessageEvent};

pub fn get_plain_quote_fallback(original_message: &RoomMessageEvent) -> String {
    match &original_message.content.msgtype {
        MessageType::Audio(_) => {
            format!("> <{}> sent an audio file.", original_message.sender)
        }
        MessageType::Emote(content) => {
            format!("> * <{}> {}", original_message.sender, content.body)
        }
        MessageType::File(_) => {
            format!("> <{}> sent a file.", original_message.sender)
        }
        MessageType::Image(_) => {
            format!("> <{}> sent an image.", original_message.sender)
        }
        MessageType::Location(content) => {
            format!("> <{}> {}", original_message.sender, content.body)
        }
        MessageType::Notice(content) => {
            format!("> <{}> {}", original_message.sender, content.body)
        }
        MessageType::ServerNotice(content) => {
            format!("> <{}> {}", original_message.sender, content.body)
        }
        MessageType::Text(content) => {
            format!("> <{}> {}", original_message.sender, content.body)
        }
        MessageType::Video(_) => {
            format!("> <{}> sent a video.", original_message.sender)
        }
        MessageType::_Custom(content) => {
            format!("> <{}> {}", original_message.sender, content.body)
        }
        #[cfg(feature = "unstable-pre-spec")]
        MessageType::VerificationRequest(content) => {
            format!("> <{}> {}", original_message.sender, content.body)
        }
    }
    .replace('\n', "\n> ")
}

#[allow(clippy::nonstandard_macro_braces)]
pub fn get_html_quote_fallback(original_message: &RoomMessageEvent) -> String {
    match &original_message.content.msgtype {
        MessageType::Audio(_) => {
            formatdoc!(
                "
                <mx-reply>
                    <blockquote>
                        <a href=\"https://matrix.to/#/{room_id}/{event_id}\">In reply to</a>
                        <a href=\"https://matrix.to/#/{sender}\">{sender}</a>
                        <br />
                        sent an audio file.
                    </blockquote>
                </mx-reply>
                ",
                room_id = original_message.room_id,
                event_id = original_message.event_id,
                sender = original_message.sender,
            )
        }
        MessageType::Emote(content) => {
            formatdoc!(
                "
                <mx-reply>
                    <blockquote>
                        <a href=\"https://matrix.to/#/{room_id}/{event_id}\">In reply to</a>
                        * <a href=\"https://matrix.to/#/{sender}\">{sender}</a>
                        <br />
                        {body}
                    </blockquote>
                </mx-reply>
                ",
                room_id = original_message.room_id,
                event_id = original_message.event_id,
                sender = original_message.sender,
                body = formatted_or_plain_body(&content.formatted, &content.body),
            )
        }
        MessageType::File(_) => {
            formatdoc!(
                "
                <mx-reply>
                    <blockquote>
                        <a href=\"https://matrix.to/#/{room_id}/{event_id}\">In reply to</a>
                        <a href=\"https://matrix.to/#/{sender}\">{sender}</a>
                        <br />
                        sent a file.
                    </blockquote>
                </mx-reply>
                ",
                room_id = original_message.room_id,
                event_id = original_message.event_id,
                sender = original_message.sender,
            )
        }
        MessageType::Image(_) => {
            formatdoc!(
                "
                <mx-reply>
                    <blockquote>
                        <a href=\"https://matrix.to/#/{room_id}/{event_id}\">In reply to</a>
                        <a href=\"https://matrix.to/#/{sender}\">{sender}</a>
                        <br />
                        sent an image.
                    </blockquote>
                </mx-reply>
                ",
                room_id = original_message.room_id,
                event_id = original_message.event_id,
                sender = original_message.sender,
            )
        }
        MessageType::Location(_) => {
            formatdoc!(
                "
                <mx-reply>
                    <blockquote>
                        <a href=\"https://matrix.to/#/{room_id}/{event_id}\">In reply to</a>
                        <a href=\"https://matrix.to/#/{sender}\">{sender}</a>
                        <br />
                        sent a location.
                    </blockquote>
                </mx-reply>
                ",
                room_id = original_message.room_id,
                event_id = original_message.event_id,
                sender = original_message.sender,
            )
        }
        MessageType::Notice(content) => {
            formatdoc!(
                "
                <mx-reply>
                    <blockquote>
                        <a href=\"https://matrix.to/#/{room_id}/{event_id}\">In reply to</a>
                        <a href=\"https://matrix.to/#/{sender}\">{sender}</a>
                        <br />
                        {body}
                    </blockquote>
                </mx-reply>
                ",
                room_id = original_message.room_id,
                event_id = original_message.event_id,
                sender = original_message.sender,
                body = formatted_or_plain_body(&content.formatted, &content.body),
            )
        }
        MessageType::ServerNotice(content) => {
            formatdoc!(
                "
                <mx-reply>
                    <blockquote>
                        <a href=\"https://matrix.to/#/{room_id}/{event_id}\">In reply to</a>
                        <a href=\"https://matrix.to/#/{sender}\">{sender}</a>
                        <br />
                        {body}
                    </blockquote>
                </mx-reply>
                ",
                room_id = original_message.room_id,
                event_id = original_message.event_id,
                sender = original_message.sender,
                body = content.body,
            )
        }
        MessageType::Text(content) => {
            formatdoc!(
                "
                <mx-reply>
                    <blockquote>
                        <a href=\"https://matrix.to/#/{room_id}/{event_id}\">In reply to</a>
                        <a href=\"https://matrix.to/#/{sender}\">{sender}</a>
                        <br />
                        {body}
                    </blockquote>
                </mx-reply>
                ",
                room_id = original_message.room_id,
                event_id = original_message.event_id,
                sender = original_message.sender,
                body = formatted_or_plain_body(&content.formatted, &content.body),
            )
        }
        MessageType::Video(_) => {
            formatdoc!(
                "
                <mx-reply>
                    <blockquote>
                        <a href=\"https://matrix.to/#/{room_id}/{event_id}\">In reply to</a>
                        <a href=\"https://matrix.to/#/{sender}\">{sender}</a>
                        <br />
                        sent a video.
                    </blockquote>
                </mx-reply>
                ",
                room_id = original_message.room_id,
                event_id = original_message.event_id,
                sender = original_message.sender,
            )
        }
        MessageType::_Custom(content) => {
            formatdoc!(
                "
                <mx-reply>
                    <blockquote>
                        <a href=\"https://matrix.to/#/{room_id}/{event_id}\">In reply to</a>
                        <a href=\"https://matrix.to/#/{sender}\">{sender}</a>
                        <br />
                        {body}
                    </blockquote>
                </mx-reply>
                ",
                room_id = original_message.room_id,
                event_id = original_message.event_id,
                sender = original_message.sender,
                body = content.body,
            )
        }
        #[cfg(feature = "unstable-pre-spec")]
        MessageType::VerificationRequest(content) => {
            formatdoc!(
                "
                <mx-reply>
                    <blockquote>
                        <a href=\"https://matrix.to/#/{server_name}/{event_id}\">In reply to</a>
                        <a href=\"https://matrix.to/#/{sender}\">{sender}</a>
                        <br />
                        {body}
                    </blockquote>
                </mx-reply>
                ",
                server_name = original_message.room_id,
                event_id = original_message.event_id,
                sender = original_message.sender,
                body = content.body,
            )
        }
    }
}

fn formatted_or_plain_body<'a>(formatted: &'a Option<FormattedBody>, body: &'a str) -> &'a str {
    if let Some(formatted_body) = formatted {
        &formatted_body.body
    } else {
        body
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use ruma_identifiers::{EventId, RoomId, UserId};

    use super::RoomMessageEvent;
    use crate::room::message::RoomMessageEventContent;

    #[test]
    fn plain_quote_fallback_multiline() {
        let sender = UserId::try_from("@alice:example.com").unwrap();
        assert_eq!(
            super::get_plain_quote_fallback(&RoomMessageEvent {
                content: RoomMessageEventContent::text_plain("multi\nline"),
                event_id: EventId::new(sender.server_name()),
                sender,
                origin_server_ts: ruma_common::MilliSecondsSinceUnixEpoch::now(),
                room_id: RoomId::try_from("!n8f893n9:example.com").unwrap(),
                unsigned: crate::Unsigned::new(),
            }),
            "> <@alice:example.com> multi\n> line"
        );
    }
}
