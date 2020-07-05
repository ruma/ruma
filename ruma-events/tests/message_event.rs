use std::{
    convert::TryFrom,
    time::{Duration, UNIX_EPOCH},
};

use js_int::UInt;
use maplit::btreemap;
use matches::assert_matches;
use ruma_events::{
    call::{answer::AnswerEventContent, SessionDescription, SessionDescriptionType},
    room::{
        redaction::{RedactedContent, RedactionEvent, RedactionEventContent},
        ImageInfo, ThumbnailInfo,
    },
    sticker::StickerEventContent,
    AnyMessageEventContent, AnyMessageEventStub, AnyRoomEventStub, EventJson, MessageEvent,
    MessageEventStub, UnsignedData,
};
use ruma_identifiers::{EventId, RoomId, UserId};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn redaction_serialize() {
    let left_over_keys = btreemap! {
        "None".to_string() => json!("of message events keys survive"),
    };
    let redacted = MessageEventStub {
        content: RedactedContent {
            event_type: "m.room.message".to_string(),
            reason: Some("who cares".to_string()),
            left_over_keys,
        },
        event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
        sender: UserId::try_from("@carl:example.com").unwrap(),
        unsigned: UnsignedData::default(),
    };

    let expected = json!({
      "content": {
        "None": "of message events keys survive",
        "reason": "who cares"
      },
      "event_id": "$h29iv0s8:example.com",
      "origin_server_ts": 1,
      "sender": "@carl:example.com",
      "type": "m.room.message"
    });

    let actual = to_json_value(&redacted).unwrap();
    println!("{}", serde_json::to_string_pretty(&actual).unwrap());
    assert_eq!(actual, expected);
}

#[test]
fn redaction_deserialize_any_room_stub() {
    let mut unsigned = UnsignedData::default();
    // The presence of `redacted_because` triggers the event enum (AnyRoomEventStub in this case)
    // to return early with `RedactedContent` instead of failing to deserialize itself according
    // to the event type string
    unsigned.redacted_because = Some(EventJson::from(RedactionEvent {
        content: RedactionEventContent { reason: Some("redacted because".into()) },
        redacts: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
        room_id: RoomId::try_from("!roomid:room.com").unwrap(),
        sender: UserId::try_from("@carl:example.com").unwrap(),
        unsigned: UnsignedData::default(),
    }));

    let redacted = MessageEventStub {
        content: RedactedContent {
            event_type: "m.room.message".to_string(),
            reason: Some("who cares".to_string()),
            left_over_keys: btreemap! {
                "None".to_string() => json!("of message events keys survive"),
            },
        },
        event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
        sender: UserId::try_from("@carl:example.com").unwrap(),
        unsigned,
    };

    let actual = to_json_value(&redacted).unwrap();

    assert_matches!(
        from_json_value::<EventJson<AnyRoomEventStub>>(actual)
            .unwrap()
            .deserialize()
            .unwrap(),
        AnyRoomEventStub::Message(AnyMessageEventStub::IsRedacted(MessageEventStub {
            event_id, content, unsigned, ..
        })) if event_id == redacted.event_id && content.reason == redacted.content.reason
            && unsigned.age == redacted.unsigned.age
    )
}

#[test]
fn redaction_deserialize_to_event_kind() {
    let mut unsigned = UnsignedData::default();
    unsigned.redacted_because = Some(EventJson::from(RedactionEvent {
        content: RedactionEventContent { reason: Some("redacted because".into()) },
        redacts: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
        room_id: RoomId::try_from("!roomid:room.com").unwrap(),
        sender: UserId::try_from("@carl:example.com").unwrap(),
        unsigned: UnsignedData::default(),
    }));

    let redacted = MessageEventStub {
        content: RedactedContent {
            event_type: "m.room.message".to_string(),
            reason: Some("who cares".to_string()),
            left_over_keys: btreemap! {
                "None".to_string() => json!("of message events keys survive"),
            },
        },
        event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
        sender: UserId::try_from("@carl:example.com").unwrap(),
        unsigned,
    };

    let actual = to_json_value(&redacted).unwrap();

    assert_matches!(
        from_json_value::<EventJson<MessageEventStub<RedactedContent>>>(actual)
            .unwrap()
            .deserialize()
            .unwrap(),
        MessageEventStub {
            event_id, content, unsigned, ..
        } if event_id == redacted.event_id && content.reason == redacted.content.reason
            && unsigned.age == redacted.unsigned.age
    )
}

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
