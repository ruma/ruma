use assert_matches::assert_matches;
use assign::assign;
use js_int::{uint, UInt};
use ruma_common::{
    events::{
        call::answer::CallAnswerEventContent,
        room::{ImageInfo, MediaSource, ThumbnailInfo},
        sticker::StickerEventContent,
        AnyMessageLikeEvent, AnySyncMessageLikeEvent, MessageLikeEvent,
    },
    mxc_uri, room_id,
    serde::CanBeEmpty,
    MilliSecondsSinceUnixEpoch, VoipVersionId,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn message_serialize_sticker() {
    let content = StickerEventContent::new(
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
    );

    let actual = to_json_value(&content).unwrap();

    #[cfg(not(feature = "unstable-msc3552"))]
    let expected = json!({
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
    });

    #[cfg(feature = "unstable-msc3552")]
    let expected = json!({
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
        "version": 0
    });

    let content = from_json_value::<CallAnswerEventContent>(json_data).unwrap();

    assert_eq!(content.answer.sdp, "Hello");
    assert_eq!(content.call_id, "foofoo");
    assert_eq!(content.version, VoipVersionId::V0);
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
            "version": 0
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
    assert_eq!(content.version, VoipVersionId::V0);
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

    let message_event = assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data),
        Ok(AnyMessageLikeEvent::Sticker(MessageLikeEvent::Original(message_event))) => message_event
    );

    assert_eq!(message_event.event_id, "$h29iv0s8:example.com");
    assert_eq!(message_event.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(message_event.room_id, "!roomid:room.com");
    assert_eq!(message_event.sender, "@carl:example.com");
    assert!(message_event.unsigned.is_empty());

    let content = message_event.content;
    assert_eq!(content.body, "Hello");
    assert_eq!(content.info.height, Some(uint!(423)));
    assert_eq!(content.info.width, Some(uint!(1011)));
    assert_eq!(content.info.mimetype.as_deref(), Some("image/png"));
    assert_eq!(content.info.size, Some(uint!(84242)));
    assert_eq!(content.url, "mxc://matrix.org/jxPXTKpyydzdHJkdFNZjTZrD");

    let thumbnail_url = assert_matches!(
        content.info.thumbnail_source,
        Some(MediaSource::Plain(thumbnail_url)) => thumbnail_url
    );
    assert_eq!(thumbnail_url, "mxc://matrix.org/irnsNRS2879");
    let thumbnail_info = content.info.thumbnail_info.unwrap();
    assert_eq!(thumbnail_info.width, Some(uint!(800)));
    assert_eq!(thumbnail_info.height, Some(uint!(334)));
    assert_eq!(thumbnail_info.mimetype.as_deref(), Some("image/png"));
    assert_eq!(thumbnail_info.size, Some(uint!(82595)));
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
            "version": 0
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
    assert_eq!(content.version, VoipVersionId::V0);
}
