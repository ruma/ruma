//! An enum that represents any message event. A message event is represented by
//! a parameterized struct allowing more flexibility in whats being sent.

use std::{
    convert::TryFrom,
    time::{SystemTime, UNIX_EPOCH},
};

use js_int::UInt;
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{
    ser::{Error, SerializeStruct},
    Serialize, Serializer,
};

use crate::{MessageEventContent, RawEventContent, RoomEventContent, TryFromRaw, UnsignedData};
use ruma_events_macros::{event_content_collection, Event};

event_content_collection! {
    /// A message event.
    name: AnyMessageEventContent,
    events: [
        "m.call.answer",
        "m.call.invite",
        "m.call.hangup",
        "m.call.candidates",
        "m.sticker",
    ]
}

/// Message event.
#[derive(Clone, Debug, Event)]
pub struct MessageEvent<C: MessageEventContent>
where
    C::Raw: RawEventContent,
{
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: EventId,

    /// Contains the fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: SystemTime,

    /// The ID of the room associated with this event.
    pub room_id: RoomId,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: UnsignedData,
}

#[cfg(test)]
mod tests {
    use std::{
        convert::TryFrom,
        time::{Duration, UNIX_EPOCH},
    };

    use js_int::UInt;
    use matches::assert_matches;
    use ruma_identifiers::{EventId, RoomAliasId, RoomId, UserId};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{AnyMessageEventContent, MessageEvent};
    use crate::{
        call::{answer::AnswerEventContent, SessionDescription, SessionDescriptionType},
        room::{ImageInfo, ThumbnailInfo},
        sticker::StickerEventContent,
        EventJson, UnsignedData,
    };

    #[test]
    fn message_serialize_sticker() {
        let aliases_event = MessageEvent {
            content: AnyMessageEventContent::Sticker(StickerEventContent {
                body: "Hello".into(),
                info: ImageInfo {
                    height: UInt::new(423),
                    width: UInt::new(1011),
                    mimetype: Some("image/png".into()),
                    size: UInt::new(84242),
                    thumbnail_info: Some(Box::new(ThumbnailInfo {
                        width: UInt::new(800),
                        height: UInt::new(334),
                        mimetype: Some("image/png".into()),
                        size: UInt::new(82595),
                    })),
                    thumbnail_url: Some("mxc://matrix.org".into()),
                    thumbnail_file: None,
                },
                url: "http://www.matrix.org".into(),
            }),
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
            room_id: RoomId::try_from("!roomid:room.com").unwrap(),
            sender: UserId::try_from("@carl:example.com").unwrap(),
            unsigned: UnsignedData::default(),
        };

        let actual = to_json_value(&aliases_event).unwrap();
        let expected = json!({
            "content": {
                "body": "Hello",
                "info": {
                    "h": 423,
                    "mimetype": "image/png",
                    "size": 84242,
                    "thumbnail_info": {
                      "h": 334,
                      "mimetype": "image/png",
                      "size": 82595,
                      "w": 800
                    },
                    "thumbnail_url": "mxc://matrix.org",
                    "w": 1011
                  },
                "url": "http://www.matrix.org"
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!roomid:room.com",
            "sender": "@carl:example.com",
            "type": "m.sticker",
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn deserialize_message_call_answer_content() {
        let json_data = json!({
            "answer": {
                "type": "answer",
                "sdp": "Hello"
            },
            "call_id": "foofoo",
            "version": 1
        });

        assert_matches!(
            from_json_value::<EventJson<AnyMessageEventContent>>(json_data)
                .unwrap()
                .deserialize_content("m.call.answer")
                .unwrap(),
            AnyMessageEventContent::CallAnswer(AnswerEventContent {
                answer: SessionDescription {
                    session_type: SessionDescriptionType::Answer,
                    sdp,
                },
                call_id,
                version,
            }) if sdp == "Hello" && call_id == "foofoo" && version == UInt::new(1).unwrap()
        );
    }

    #[test]
    fn deserialize_message_call_answer() {
        let json_data = json!({
            "content": {
                "answer": {
                    "type": "answer",
                    "sdp": "Hello"
                },
                "call_id": "foofoo",
                "version": 1
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!roomid:room.com",
            "sender": "@carl:example.com",
            "type": "m.call.answer"
        });

        assert_matches!(
            from_json_value::<EventJson<MessageEvent<AnyMessageEventContent>>>(json_data)
                .unwrap()
                .deserialize()
                .unwrap(),
            MessageEvent {
                content: AnyMessageEventContent::CallAnswer(AnswerEventContent {
                    answer: SessionDescription {
                        session_type: SessionDescriptionType::Answer,
                        sdp,
                    },
                    call_id,
                    version,
                }),
                event_id,
                origin_server_ts,
                room_id,
                sender,
                unsigned,
            } if sdp == "Hello" && call_id == "foofoo" && version == UInt::new(1).unwrap()
                && event_id == EventId::try_from("$h29iv0s8:example.com").unwrap()
                && origin_server_ts == UNIX_EPOCH + Duration::from_millis(1)
                && room_id == RoomId::try_from("!roomid:room.com").unwrap()
                && sender == UserId::try_from("@carl:example.com").unwrap()
                && unsigned.is_empty()
        );
    }

    #[test]
    fn deserialize_message_sticker() {
        let json_data = json!({
            "content": {
                "body": "Hello",
                "info": {
                    "h": 423,
                    "mimetype": "image/png",
                    "size": 84242,
                    "thumbnail_info": {
                      "h": 334,
                      "mimetype": "image/png",
                      "size": 82595,
                      "w": 800
                    },
                    "thumbnail_url": "mxc://matrix.org",
                    "w": 1011
                  },
                "url": "http://www.matrix.org"
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!roomid:room.com",
            "sender": "@carl:example.com",
            "type": "m.sticker"
        });

        assert_matches!(
            from_json_value::<EventJson<MessageEvent<AnyMessageEventContent>>>(json_data)
                .unwrap()
                .deserialize()
                .unwrap(),
            MessageEvent {
                content: AnyMessageEventContent::Sticker(StickerEventContent {
                    body,
                    info: ImageInfo {
                        height,
                        width,
                        mimetype: Some(mimetype),
                        size,
                        thumbnail_info: Some(thumbnail_info),
                        thumbnail_url: Some(thumbnail_url),
                        thumbnail_file: None,
                    },
                    url,
                }),
                event_id,
                origin_server_ts,
                room_id,
                sender,
                unsigned
            } if event_id == EventId::try_from("$h29iv0s8:example.com").unwrap()
                && body == "Hello"
                && origin_server_ts == UNIX_EPOCH + Duration::from_millis(1)
                && room_id == RoomId::try_from("!roomid:room.com").unwrap()
                && sender == UserId::try_from("@carl:example.com").unwrap()
                && height == UInt::new(423)
                && width == UInt::new(1011)
                && mimetype == "image/png"
                && size == UInt::new(84242)
                && thumbnail_url == "mxc://matrix.org"
                && matches!(
                    thumbnail_info.as_ref(),
                    ThumbnailInfo {
                        width: thumb_width,
                        height: thumb_height,
                        mimetype: thumb_mimetype,
                        size: thumb_size,
                    } if *thumb_width == UInt::new(800)
                        && *thumb_height == UInt::new(334)
                        && *thumb_mimetype == Some("image/png".to_string())
                        && *thumb_size == UInt::new(82595)
                )
                && url == "http://www.matrix.org"
                && unsigned.is_empty()
        );
    }
}
