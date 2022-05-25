use assert_matches::assert_matches;
use assign::assign;
use js_int::{uint, UInt};
#[cfg(feature = "unstable-msc2746")]
use ruma_common::events::call::CallVersion;
use ruma_common::{
    event_id,
    events::{
        room::{ImageInfo, MediaSource, ThumbnailInfo},
        sticker::StickerEventContent,
        AnyMessageLikeEvent, AnyMessageLikeEventContent, AnySyncMessageLikeEvent, MessageLikeEvent,
        MessageLikeEventType, MessageLikeUnsigned, OriginalMessageLikeEvent,
    },
    mxc_uri, room_id,
    serde::Raw,
    user_id, MilliSecondsSinceUnixEpoch,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn message_serialize_sticker() {
    let aliases_event = OriginalMessageLikeEvent {
        content: StickerEventContent::new(
            "Hello".into(),
            assign!(ImageInfo::new(), {
                height: UInt::new(423),
                width: UInt::new(1011),
                mimetype: Some("image/png".into()),
                size: UInt::new(84242),
                thumbnail_info: Some(Box::new(assign!(ThumbnailInfo::new(), {
                    width: UInt::new(800),
                    height: UInt::new(334),
                    mimetype: Some("image/png".into()),
                    size: UInt::new(82595),
                }))),
                thumbnail_source: Some(MediaSource::Plain(mxc_uri!("mxc://matrix.org/irsns989Rrsn").to_owned())),
            }),
            mxc_uri!("mxc://matrix.org/rnsldl8srs98IRrs").to_owned(),
        ),
        event_id: event_id!("$h29iv0s8:example.com").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
        room_id: room_id!("!roomid:room.com").to_owned(),
        sender: user_id!("@carl:example.com").to_owned(),
        unsigned: MessageLikeUnsigned::default(),
    };

    let actual = to_json_value(&aliases_event).unwrap();

    #[cfg(not(feature = "unstable-msc3552"))]
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
                "thumbnail_url": "mxc://matrix.org/irsns989Rrsn",
                "w": 1011
              },
            "url": "mxc://matrix.org/rnsldl8srs98IRrs"
        },
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "room_id": "!roomid:room.com",
        "sender": "@carl:example.com",
        "type": "m.sticker",
    });

    #[cfg(feature = "unstable-msc3552")]
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
                "thumbnail_url": "mxc://matrix.org/irsns989Rrsn",
                "w": 1011
              },
            "url": "mxc://matrix.org/rnsldl8srs98IRrs",
            "org.matrix.msc1767.text": "Hello",
            "org.matrix.msc1767.file": {
                "url": "mxc://matrix.org/rnsldl8srs98IRrs",
                "mimetype": "image/png",
                "size": 84242,
            },
            "org.matrix.msc1767.image": {
                "height": 423,
                "width": 1011,
            },
            "org.matrix.msc1767.thumbnail": [
                {
                    "url": "mxc://matrix.org/irsns989Rrsn",
                    "mimetype": "image/png",
                    "size": 82595,
                    "height": 334,
                    "width": 800,
                }
            ],
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

    let content = assert_matches!(
        from_json_value::<Raw<AnyMessageLikeEventContent>>(json_data)
            .unwrap()
            .deserialize_content(MessageLikeEventType::CallAnswer)
            .unwrap(),
        AnyMessageLikeEventContent::CallAnswer(content) => content
    );

    assert_eq!(content.answer.sdp, "Hello");
    assert_eq!(content.call_id, "foofoo");
    #[cfg(not(feature = "unstable-msc2746"))]
    assert_eq!(content.version, uint!(1));
    #[cfg(feature = "unstable-msc2746")]
    assert_eq!(content.version, CallVersion::Stable(uint!(1)));
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

    let message_event = assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::CallAnswer(MessageLikeEvent::Original(message_event)) => message_event
    );
    assert_eq!(message_event.event_id, "$h29iv0s8:example.com");
    assert_eq!(message_event.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(message_event.room_id, "!roomid:room.com");
    assert_eq!(message_event.sender, "@carl:example.com");
    assert!(message_event.unsigned.is_empty());

    let content = message_event.content;
    assert_eq!(content.answer.sdp, "Hello");
    assert_eq!(content.call_id, "foofoo");
    #[cfg(not(feature = "unstable-msc2746"))]
    assert_eq!(content.version, uint!(1));
    #[cfg(feature = "unstable-msc2746")]
    assert_eq!(content.version, CallVersion::Stable(uint!(1)));
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
                "thumbnail_url": "mxc://matrix.org/irnsNRS2879",
                "w": 1011
              },
            "url": "mxc://matrix.org/jxPXTKpyydzdHJkdFNZjTZrD"
        },
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "room_id": "!roomid:room.com",
        "sender": "@carl:example.com",
        "type": "m.sticker"
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::Sticker(MessageLikeEvent::Original(OriginalMessageLikeEvent {
            content: StickerEventContent {
                body,
                info: ImageInfo {
                    height,
                    width,
                    mimetype: Some(mimetype),
                    size,
                    thumbnail_info: Some(thumbnail_info),
                    thumbnail_source: Some(MediaSource::Plain(thumbnail_url)),
                    #[cfg(feature = "unstable-msc2448")]
                    blurhash: None,
                    ..
                },
                url,
                ..
            },
            event_id,
            origin_server_ts,
            room_id,
            sender,
            unsigned
        })) if event_id == event_id!("$h29iv0s8:example.com")
            && body == "Hello"
            && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(1))
            && room_id == room_id!("!roomid:room.com")
            && sender == user_id!("@carl:example.com")
            && height == UInt::new(423)
            && width == UInt::new(1011)
            && mimetype == "image/png"
            && size == UInt::new(84242)
            && thumbnail_url == "mxc://matrix.org/irnsNRS2879"
            && matches!(
                thumbnail_info.as_ref(),
                ThumbnailInfo {
                    width: thumb_width,
                    height: thumb_height,
                    mimetype: thumb_mimetype,
                    size: thumb_size,
                    ..
                } if *thumb_width == UInt::new(800)
                    && *thumb_height == UInt::new(334)
                    && *thumb_mimetype == Some("image/png".into())
                    && *thumb_size == UInt::new(82595)
            )
            && url == "mxc://matrix.org/jxPXTKpyydzdHJkdFNZjTZrD"
            && unsigned.is_empty()
    );
}

#[test]
fn deserialize_message_then_convert_to_full() {
    let rid = room_id!("!roomid:room.com");
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
        "sender": "@carl:example.com",
        "type": "m.call.answer"
    });

    let sync_ev: AnySyncMessageLikeEvent = from_json_value(json_data).unwrap();

    let message_event = assert_matches!(
        sync_ev.into_full_event(rid.to_owned()),
        AnyMessageLikeEvent::CallAnswer(MessageLikeEvent::Original(message_event)) => message_event
    );
    assert_eq!(message_event.event_id, "$h29iv0s8:example.com");
    assert_eq!(message_event.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(message_event.room_id, "!roomid:room.com");
    assert_eq!(message_event.sender, "@carl:example.com");
    assert!(message_event.unsigned.is_empty());

    let content = message_event.content;
    assert_eq!(content.answer.sdp, "Hello");
    assert_eq!(content.call_id, "foofoo");
    #[cfg(not(feature = "unstable-msc2746"))]
    assert_eq!(content.version, uint!(1));
    #[cfg(feature = "unstable-msc2746")]
    assert_eq!(content.version, CallVersion::Stable(uint!(1)));
}
