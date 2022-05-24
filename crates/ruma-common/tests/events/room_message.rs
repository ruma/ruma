use std::borrow::Cow;

use assert_matches::assert_matches;
#[cfg(not(feature = "unstable-msc1767"))]
use assign::assign;
use js_int::uint;
#[cfg(not(feature = "unstable-msc1767"))]
use ruma_common::events::room::message::InReplyTo;
#[cfg(any(feature = "unstable-msc2676", not(feature = "unstable-msc1767")))]
use ruma_common::events::room::message::Relation;
use ruma_common::{
    event_id,
    events::{
        key::verification::VerificationMethod,
        room::{
            message::{
                AudioMessageEventContent, KeyVerificationRequestEventContent, MessageType,
                OriginalRoomMessageEvent, RoomMessageEventContent, TextMessageEventContent,
            },
            MediaSource,
        },
        MessageLikeUnsigned,
    },
    mxc_uri, room_id, user_id, MilliSecondsSinceUnixEpoch, OwnedDeviceId,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

macro_rules! json_object {
    ( $($key:expr => $value:expr),* $(,)? ) => {
        {
            let mut _map = serde_json::Map::<String, serde_json::Value>::new();
            $(
                let _ = _map.insert($key, $value);
            )*
            _map
        }
    };
}

#[test]
fn serialization() {
    let ev = OriginalRoomMessageEvent {
        content: RoomMessageEventContent::new(MessageType::Audio(AudioMessageEventContent::plain(
            "test".into(),
            mxc_uri!("mxc://example.org/ffed755USFFxlgbQYZGtryd").to_owned(),
            None,
        ))),
        event_id: event_id!("$143273582443PhrSn:example.org").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(10_000)),
        room_id: room_id!("!testroomid:example.org").to_owned(),
        sender: user_id!("@user:example.org").to_owned(),
        unsigned: MessageLikeUnsigned::default(),
    };

    #[cfg(not(feature = "unstable-msc3246"))]
    assert_eq!(
        to_json_value(ev).unwrap(),
        json!({
            "type": "m.room.message",
            "event_id": "$143273582443PhrSn:example.org",
            "origin_server_ts": 10_000,
            "room_id": "!testroomid:example.org",
            "sender": "@user:example.org",
            "content": {
                "body": "test",
                "msgtype": "m.audio",
                "url": "mxc://example.org/ffed755USFFxlgbQYZGtryd",
            }
        })
    );

    #[cfg(feature = "unstable-msc3246")]
    assert_eq!(
        to_json_value(ev).unwrap(),
        json!({
            "type": "m.room.message",
            "event_id": "$143273582443PhrSn:example.org",
            "origin_server_ts": 10_000,
            "room_id": "!testroomid:example.org",
            "sender": "@user:example.org",
            "content": {
                "body": "test",
                "msgtype": "m.audio",
                "url": "mxc://example.org/ffed755USFFxlgbQYZGtryd",
                "org.matrix.msc1767.text": "test",
                "org.matrix.msc1767.file": {
                    "url": "mxc://example.org/ffed755USFFxlgbQYZGtryd",
                },
                "org.matrix.msc1767.audio": {},
            }
        })
    );
}

#[test]
fn content_serialization() {
    let message_event_content =
        RoomMessageEventContent::new(MessageType::Audio(AudioMessageEventContent::plain(
            "test".into(),
            mxc_uri!("mxc://example.org/ffed755USFFxlgbQYZGtryd").to_owned(),
            None,
        )));

    #[cfg(not(feature = "unstable-msc3246"))]
    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "test",
            "msgtype": "m.audio",
            "url": "mxc://example.org/ffed755USFFxlgbQYZGtryd"
        })
    );

    #[cfg(feature = "unstable-msc3246")]
    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "test",
            "msgtype": "m.audio",
            "url": "mxc://example.org/ffed755USFFxlgbQYZGtryd",
            "org.matrix.msc1767.text": "test",
            "org.matrix.msc1767.file": {
                "url": "mxc://example.org/ffed755USFFxlgbQYZGtryd",
            },
            "org.matrix.msc1767.audio": {},
        })
    );
}

#[test]
fn custom_msgtype_serialization() {
    let json_data = json_object! {
        "custom_field".into() => json!("baba"),
        "another_one".into() => json!("abab"),
    };
    let custom_msgtype =
        MessageType::new("my_custom_msgtype", "my message body".into(), json_data).unwrap();

    assert_eq!(
        to_json_value(&custom_msgtype).unwrap(),
        json!({
            "msgtype": "my_custom_msgtype",
            "body": "my message body",
            "custom_field": "baba",
            "another_one": "abab",
        })
    );
}

#[test]
fn custom_content_deserialization() {
    let json_data = json!({
        "msgtype": "my_custom_msgtype",
        "body": "my custom message",
        "custom_field": "baba",
        "another_one": "abab",
    });

    let expected_json_data = json_object! {
        "custom_field".into() => json!("baba"),
        "another_one".into() => json!("abab"),
    };

    let custom_event: MessageType = from_json_value(json_data).unwrap();

    assert_eq!(custom_event.msgtype(), "my_custom_msgtype");
    assert_eq!(custom_event.body(), "my custom message");
    assert_eq!(custom_event.data(), Cow::Owned(expected_json_data));
}

#[test]
fn formatted_body_serialization() {
    let message_event_content =
        RoomMessageEventContent::text_html("Hello, World!", "Hello, <em>World</em>!");

    #[cfg(not(feature = "unstable-msc1767"))]
    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "Hello, World!",
            "msgtype": "m.text",
            "format": "org.matrix.custom.html",
            "formatted_body": "Hello, <em>World</em>!",
        })
    );

    #[cfg(feature = "unstable-msc1767")]
    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "Hello, World!",
            "msgtype": "m.text",
            "format": "org.matrix.custom.html",
            "formatted_body": "Hello, <em>World</em>!",
            "org.matrix.msc1767.message": [
                { "body": "Hello, <em>World</em>!", "mimetype": "text/html" },
                { "body": "Hello, World!", "mimetype": "text/plain" },
            ],
        })
    );
}

#[test]
fn plain_text_content_serialization() {
    let message_event_content =
        RoomMessageEventContent::text_plain("> <@test:example.com> test\n\ntest reply");

    #[cfg(not(feature = "unstable-msc1767"))]
    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "> <@test:example.com> test\n\ntest reply",
            "msgtype": "m.text"
        })
    );

    #[cfg(feature = "unstable-msc1767")]
    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "> <@test:example.com> test\n\ntest reply",
            "msgtype": "m.text",
            "org.matrix.msc1767.text": "> <@test:example.com> test\n\ntest reply",
        })
    );
}

#[test]
#[cfg(feature = "markdown")]
fn markdown_content_serialization() {
    let formatted_message = RoomMessageEventContent::new(MessageType::Text(
        TextMessageEventContent::markdown("Testing **bold** and _italic_!"),
    ));

    #[cfg(not(feature = "unstable-msc1767"))]
    assert_eq!(
        to_json_value(&formatted_message).unwrap(),
        json!({
            "body": "Testing **bold** and _italic_!",
            "formatted_body": "<p>Testing <strong>bold</strong> and <em>italic</em>!</p>\n",
            "format": "org.matrix.custom.html",
            "msgtype": "m.text"
        })
    );

    #[cfg(feature = "unstable-msc1767")]
    assert_eq!(
        to_json_value(&formatted_message).unwrap(),
        json!({
            "body": "Testing **bold** and _italic_!",
            "formatted_body": "<p>Testing <strong>bold</strong> and <em>italic</em>!</p>\n",
            "format": "org.matrix.custom.html",
            "msgtype": "m.text",
            "org.matrix.msc1767.message": [
                { "body": "<p>Testing <strong>bold</strong> and <em>italic</em>!</p>\n", "mimetype": "text/html" },
                { "body": "Testing **bold** and _italic_!", "mimetype": "text/plain" },
            ],
        })
    );

    let plain_message_simple = RoomMessageEventContent::new(MessageType::Text(
        TextMessageEventContent::markdown("Testing a simple phrase…"),
    ));

    #[cfg(not(feature = "unstable-msc1767"))]
    assert_eq!(
        to_json_value(&plain_message_simple).unwrap(),
        json!({
            "body": "Testing a simple phrase…",
            "msgtype": "m.text"
        })
    );

    #[cfg(feature = "unstable-msc1767")]
    assert_eq!(
        to_json_value(&plain_message_simple).unwrap(),
        json!({
            "body": "Testing a simple phrase…",
            "msgtype": "m.text",
            "org.matrix.msc1767.text": "Testing a simple phrase…",
        })
    );

    let plain_message_paragraphs = RoomMessageEventContent::new(MessageType::Text(
        TextMessageEventContent::markdown("Testing\n\nSeveral\n\nParagraphs."),
    ));

    #[cfg(not(feature = "unstable-msc1767"))]
    assert_eq!(
        to_json_value(&plain_message_paragraphs).unwrap(),
        json!({
            "body": "Testing\n\nSeveral\n\nParagraphs.",
            "formatted_body": "<p>Testing</p>\n<p>Several</p>\n<p>Paragraphs.</p>\n",
            "format": "org.matrix.custom.html",
            "msgtype": "m.text"
        })
    );

    #[cfg(feature = "unstable-msc1767")]
    assert_eq!(
        to_json_value(&plain_message_paragraphs).unwrap(),
        json!({
            "body": "Testing\n\nSeveral\n\nParagraphs.",
            "formatted_body": "<p>Testing</p>\n<p>Several</p>\n<p>Paragraphs.</p>\n",
            "format": "org.matrix.custom.html",
            "msgtype": "m.text",
            "org.matrix.msc1767.message": [
                { "body": "<p>Testing</p>\n<p>Several</p>\n<p>Paragraphs.</p>\n", "mimetype": "text/html" },
                { "body": "Testing\n\nSeveral\n\nParagraphs.", "mimetype": "text/plain" },
            ],
        })
    );
}

#[test]
#[cfg(not(feature = "unstable-msc1767"))]
fn relates_to_content_serialization() {
    let message_event_content =
        assign!(RoomMessageEventContent::text_plain("> <@test:example.com> test\n\ntest reply"), {
            relates_to: Some(Relation::Reply {
                in_reply_to: InReplyTo::new(
                    event_id!("$15827405538098VGFWH:example.com").to_owned(),
                ),
            }),
        });

    let json_data = json!({
        "body": "> <@test:example.com> test\n\ntest reply",
        "msgtype": "m.text",
        "m.relates_to": {
            "m.in_reply_to": {
                "event_id": "$15827405538098VGFWH:example.com"
            }
        }
    });

    assert_eq!(to_json_value(&message_event_content).unwrap(), json_data);
}

#[test]
#[cfg(not(feature = "unstable-msc2676"))]
fn edit_deserialization_061() {
    let json_data = json!({
        "body": "s/foo/bar",
        "msgtype": "m.text",
        "m.relates_to": {
            "rel_type": "m.replace",
            "event_id": event_id!("$1598361704261elfgc:localhost"),
        },
        "m.new_content": {
            "body": "bar",
        },
    });

    assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data).unwrap(),
        RoomMessageEventContent {
            msgtype: MessageType::Text(TextMessageEventContent {
                body,
                formatted: None,
                ..
            }),
            relates_to: Some(_),
            ..
        } if body == "s/foo/bar"
    );
}

#[test]
#[cfg(feature = "unstable-msc2676")]
fn edit_deserialization_future() {
    use ruma_common::events::room::message::Replacement;

    let ev_id = event_id!("$1598361704261elfgc:localhost");
    let json_data = json!({
        "body": "s/foo/bar",
        "msgtype": "m.text",
        "m.relates_to": {
            "rel_type": "m.replace",
            "event_id": ev_id,
        },
        "m.new_content": {
            "body": "bar",
            "msgtype": "m.text",
        },
    });

    assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data).unwrap(),
        RoomMessageEventContent {
            msgtype: MessageType::Text(TextMessageEventContent {
                body,
                formatted: None,
                ..
            }),
            relates_to: Some(Relation::Replacement(Replacement { event_id, new_content, .. })),
            ..
        } if body == "s/foo/bar"
            && event_id == ev_id
            && matches!(
                &*new_content,
                RoomMessageEventContent {
                    msgtype: MessageType::Text(TextMessageEventContent {
                        body,
                        formatted: None,
                        ..
                    }),
                    ..
                } if body == "bar"
            )
    );
}

#[test]
fn verification_request_deserialization() {
    let user_id = user_id!("@example2:localhost");
    let device_id: OwnedDeviceId = "XOWLHHFSWM".into();

    let json_data = json!({
        "body": "@example:localhost is requesting to verify your key, ...",
        "msgtype": "m.key.verification.request",
        "to": user_id,
        "from_device": device_id,
        "methods": [
            "m.sas.v1",
            "m.qr_code.show.v1",
            "m.reciprocate.v1"
        ]
    });

    assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data).unwrap(),
        RoomMessageEventContent {
            msgtype: MessageType::VerificationRequest(KeyVerificationRequestEventContent {
                body,
                to,
                from_device,
                methods,
                ..
            }),
            ..
        } if body == "@example:localhost is requesting to verify your key, ..."
            && to == user_id
            && from_device == device_id
            && methods.contains(&VerificationMethod::SasV1)
    );
}

#[test]
fn verification_request_serialization() {
    let user_id = user_id!("@example2:localhost").to_owned();
    let device_id: OwnedDeviceId = "XOWLHHFSWM".into();
    let body = "@example:localhost is requesting to verify your key, ...".to_owned();

    let methods =
        vec![VerificationMethod::SasV1, "m.qr_code.show.v1".into(), "m.reciprocate.v1".into()];

    let json_data = json!({
        "body": body,
        "msgtype": "m.key.verification.request",
        "to": user_id,
        "from_device": device_id,
        "methods": methods
    });

    let content = MessageType::VerificationRequest(KeyVerificationRequestEventContent::new(
        body, methods, device_id, user_id,
    ));

    assert_eq!(to_json_value(&content).unwrap(), json_data,);
}

#[test]
fn content_deserialization() {
    let json_data = json!({
        "body": "test",
        "msgtype": "m.audio",
        "url": "mxc://example.org/ffed755USFFxlgbQYZGtryd"
    });

    assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data)
            .unwrap(),
        RoomMessageEventContent {
            msgtype: MessageType::Audio(AudioMessageEventContent {
                body,
                info: None,
                source: MediaSource::Plain(url),
                ..
            }),
            ..
        } if body == "test" && url == "mxc://example.org/ffed755USFFxlgbQYZGtryd"
    );
}

#[test]
fn content_deserialization_failure() {
    let json_data = json!({
        "body": "test","msgtype": "m.location",
        "url": "http://example.com/audio.mp3"
    });
    assert!(from_json_value::<RoomMessageEventContent>(json_data).is_err());
}
