use indoc::formatdoc;
use ruma_common::{EventId, UserId};

use super::{
    FormattedBody, MessageType, RoomMessageEvent, RoomMessageEventContent, SyncRoomMessageEvent,
};

/// An event that can be replied to.
///
/// This trait only exists to allow the plain-text `reply` constructors on `MessageLikeEventContent`
/// to use either a [`RoomMessageEvent`] or a [`SyncRoomMessageEvent`] as the event being replied
/// to.
pub trait ReplyBaseEvent {
    #[doc(hidden)]
    fn event_id(&self) -> &EventId;

    #[doc(hidden)]
    fn sender(&self) -> &UserId;

    #[doc(hidden)]
    fn content(&self) -> &RoomMessageEventContent;
}

impl ReplyBaseEvent for RoomMessageEvent {
    fn event_id(&self) -> &EventId {
        &self.event_id
    }

    fn sender(&self) -> &UserId {
        &self.sender
    }

    fn content(&self) -> &RoomMessageEventContent {
        &self.content
    }
}

impl ReplyBaseEvent for SyncRoomMessageEvent {
    fn event_id(&self) -> &EventId {
        &self.event_id
    }

    fn sender(&self) -> &UserId {
        &self.sender
    }

    fn content(&self) -> &RoomMessageEventContent {
        &self.content
    }
}

pub fn get_plain_quote_fallback(original_message: &impl ReplyBaseEvent) -> String {
    let sender = original_message.sender();

    match &original_message.content().msgtype {
        MessageType::Audio(_) => {
            format!("> <{}> sent an audio file.", sender)
        }
        MessageType::Emote(content) => {
            format!("> * <{}> {}", sender, content.body)
        }
        MessageType::File(_) => {
            format!("> <{}> sent a file.", sender)
        }
        MessageType::Image(_) => {
            format!("> <{}> sent an image.", sender)
        }
        MessageType::Location(content) => {
            format!("> <{}> {}", sender, content.body)
        }
        MessageType::Notice(content) => {
            format!("> <{}> {}", sender, content.body)
        }
        MessageType::ServerNotice(content) => {
            format!("> <{}> {}", sender, content.body)
        }
        MessageType::Text(content) => {
            format!("> <{}> {}", sender, content.body)
        }
        MessageType::Video(_) => {
            format!("> <{}> sent a video.", sender)
        }
        MessageType::_Custom(content) => {
            format!("> <{}> {}", sender, content.body)
        }
        MessageType::VerificationRequest(content) => {
            format!("> <{}> {}", sender, content.body)
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
    use ruma_common::{event_id, room_id, user_id};

    use super::{RoomMessageEvent, RoomMessageEventContent};

    #[test]
    fn plain_quote_fallback_multiline() {
        assert_eq!(
            super::get_plain_quote_fallback(&RoomMessageEvent {
                content: RoomMessageEventContent::text_plain("multi\nline"),
                event_id: event_id!("$1598361704261elfgc:localhost").to_owned(),
                sender: user_id!("@alice:example.com").to_owned(),
                origin_server_ts: ruma_common::MilliSecondsSinceUnixEpoch::now(),
                room_id: room_id!("!n8f893n9:example.com").to_owned(),
                unsigned: crate::events::Unsigned::new(),
            }),
            "> <@alice:example.com> multi\n> line"
        );
    }
}
