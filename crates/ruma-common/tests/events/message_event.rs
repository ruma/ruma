use assign::assign;
use js_int::{uint, UInt};
use matches::assert_matches;
use ruma_common::{
    event_id,
    events::{
        call::{answer::CallAnswerEventContent, SessionDescription, SessionDescriptionType},
        room::{ImageInfo, ThumbnailInfo},
        sticker::StickerEventContent,
        AnyMessageLikeEvent, AnyMessageLikeEventContent, AnySyncMessageLikeEvent, MessageLikeEvent,
        RawExt, Unsigned,
    },
    mxc_uri, room_id, user_id, MilliSecondsSinceUnixEpoch,
};
use ruma_serde::Raw;
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn message_serialize_sticker() {
    let aliases_event = MessageLikeEvent {
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
                thumbnail_url: Some(mxc_uri!("mxc://matrix.org/irsns989Rrsn").to_owned()),
            }),
            mxc_uri!("mxc://matrix.org/rnsldl8srs98IRrs").to_owned(),
        ),
        event_id: event_id!("$h29iv0s8:example.com").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
        room_id: room_id!("!roomid:room.com").to_owned(),
        sender: user_id!("@carl:example.com").to_owned(),
        unsigned: Unsigned::default(),
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
        from_json_value::<Raw<AnyMessageLikeEventContent>>(json_data)
            .unwrap()
            .deserialize_content("m.call.answer")
            .unwrap(),
        AnyMessageLikeEventContent::CallAnswer(CallAnswerEventContent {
            answer: SessionDescription {
                session_type: SessionDescriptionType::Answer,
                sdp,
                ..
            },
            call_id,
            version,
            ..
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
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::CallAnswer(MessageLikeEvent {
            content: CallAnswerEventContent {
                answer: SessionDescription {
                    session_type: SessionDescriptionType::Answer,
                    sdp,
                    ..
                },
                call_id,
                version,
                ..
            },
            event_id,
            origin_server_ts,
            room_id,
            sender,
            unsigned,
        }) if sdp == "Hello" && call_id == "foofoo" && version == UInt::new(1).unwrap()
            && event_id == event_id!("$h29iv0s8:example.com")
            && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(1))
            && room_id == room_id!("!roomid:room.com")
            && sender == user_id!("@carl:example.com")
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
        AnyMessageLikeEvent::Sticker(MessageLikeEvent {
            content: StickerEventContent {
                body,
                info: ImageInfo {
                    height,
                    width,
                    mimetype: Some(mimetype),
                    size,
                    thumbnail_info: Some(thumbnail_info),
                    thumbnail_url: Some(thumbnail_url),
                    thumbnail_file: None,
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
        }) if event_id == event_id!("$h29iv0s8:example.com")
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

    assert_matches!(
        sync_ev.into_full_event(rid.to_owned()),
        AnyMessageLikeEvent::CallAnswer(MessageLikeEvent {
            content: CallAnswerEventContent {
                answer: SessionDescription {
                    session_type: SessionDescriptionType::Answer,
                    sdp,
                    ..
                },
                call_id,
                version,
                ..
            },
            event_id,
            origin_server_ts,
            room_id,
            sender,
            unsigned,
        }) if sdp == "Hello"
            && call_id == "foofoo"
            && version == uint!(1)
            && event_id == "$h29iv0s8:example.com"
            && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(1))
            && room_id == "!roomid:room.com"
            && sender == "@carl:example.com"
            && unsigned.is_empty()
    );
}
