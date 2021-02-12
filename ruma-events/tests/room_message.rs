use std::time::{Duration, UNIX_EPOCH};

use assign::assign;
use maplit::btreemap;
use matches::assert_matches;
#[cfg(feature = "unstable-pre-spec")]
use ruma_events::{
    key::verification::VerificationMethod, room::message::KeyVerificationRequestEventContent,
};
use ruma_events::{
    room::{
        message::{
            AudioMessageEventContent, CustomEventContent, MessageEventContent, Relation,
            TextMessageEventContent,
        },
        relationships::InReplyTo,
    },
    MessageEvent, Unsigned,
};
#[cfg(feature = "unstable-pre-spec")]
use ruma_identifiers::DeviceIdBox;
use ruma_identifiers::{event_id, room_id, user_id};
use ruma_serde::Raw;
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn serialization() {
    let ev = MessageEvent {
        content: MessageEventContent::Audio(AudioMessageEventContent {
            body: "test".into(),
            info: None,
            url: Some("http://example.com/audio.mp3".into()),
            file: None,
        }),
        event_id: event_id!("$143273582443PhrSn:example.org"),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(10_000),
        room_id: room_id!("!testroomid:example.org"),
        sender: user_id!("@user:example.org"),
        unsigned: Unsigned::default(),
    };

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
                "url": "http://example.com/audio.mp3",
            }
        })
    );
}

#[test]
fn content_serialization() {
    let message_event_content = MessageEventContent::Audio(AudioMessageEventContent {
        body: "test".into(),
        info: None,
        url: Some("http://example.com/audio.mp3".into()),
        file: None,
    });

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "test",
            "msgtype": "m.audio",
            "url": "http://example.com/audio.mp3"
        })
    );
}

#[test]
fn custom_content_serialization() {
    let json_data = btreemap! {
        "custom_field".into() => json!("baba"),
        "another_one".into() => json!("abab"),
    }
    .into_iter()
    .collect();
    let custom_event_content = MessageEventContent::_Custom(CustomEventContent {
        msgtype: "my_custom_msgtype".into(),
        data: json_data,
    });

    assert_eq!(
        to_json_value(&custom_event_content).unwrap(),
        json!({
            "msgtype": "my_custom_msgtype",
            "custom_field": "baba",
            "another_one": "abab",
        })
    );
}

#[test]
fn custom_content_deserialization() {
    let json_data = json!({
        "msgtype": "my_custom_msgtype",
        "custom_field": "baba",
        "another_one": "abab",
    });

    let expected_json_data = btreemap! {
        "custom_field".into() => json!("baba"),
        "another_one".into() => json!("abab"),
    }
    .into_iter()
    .collect();

    assert_matches!(
        from_json_value::<Raw<MessageEventContent>>(json_data)
            .unwrap()
            .deserialize()
            .unwrap(),
        MessageEventContent::_Custom(CustomEventContent {
            msgtype,
            data
        }) if msgtype == "my_custom_msgtype" && data == expected_json_data
    );
}

#[test]
fn formatted_body_serialization() {
    let message_event_content = MessageEventContent::Text(TextMessageEventContent::html(
        "Hello, World!",
        "Hello, <em>World</em>!",
    ));

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "Hello, World!",
            "msgtype": "m.text",
            "format": "org.matrix.custom.html",
            "formatted_body": "Hello, <em>World</em>!",
        })
    );
}

#[test]
fn plain_text_content_serialization() {
    let message_event_content = MessageEventContent::Text(TextMessageEventContent::plain(
        "> <@test:example.com> test\n\ntest reply",
    ));

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "> <@test:example.com> test\n\ntest reply",
            "msgtype": "m.text"
        })
    );
}

#[test]
fn relates_to_content_serialization() {
    let message_event_content = MessageEventContent::Text(
        assign!(TextMessageEventContent::plain("> <@test:example.com> test\n\ntest reply"), {
            relates_to: Some(Relation::Reply {
                in_reply_to: InReplyTo { event_id: event_id!("$15827405538098VGFWH:example.com") },
            }),
        }),
    );

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
#[cfg(not(feature = "unstable-pre-spec"))]
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
        from_json_value::<MessageEventContent>(json_data).unwrap(),
        MessageEventContent::Text(TextMessageEventContent {
            body,
            formatted: None,
            relates_to: Some(Relation::Custom(_)),
            ..
        }) if body == "s/foo/bar"
    );
}

#[test]
#[cfg(feature = "unstable-pre-spec")]
fn edit_deserialization_future() {
    use crate::room::relationships::Replacement;

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
        from_json_value::<MessageEventContent>(json_data).unwrap(),
        MessageEventContent::Text(TextMessageEventContent {
            body,
            formatted: None,
            relates_to: Some(Relation::Replacement(Replacement { event_id })),
            new_content: Some(new_content),
        }) if body == "s/foo/bar"
            && event_id == ev_id
            && matches!(
                &*new_content,
                MessageEventContent::Text(TextMessageEventContent {
                    body,
                    formatted: None,
                    relates_to: None,
                    new_content: None,
                }) if body == "bar"
            )
    );
}

#[test]
#[cfg(feature = "unstable-pre-spec")]
fn verification_request_deserialization() {
    let user_id = user_id!("@example2:localhost");
    let device_id: DeviceIdBox = "XOWLHHFSWM".into();

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
        from_json_value::<MessageEventContent>(json_data).unwrap(),
        MessageEventContent::VerificationRequest(KeyVerificationRequestEventContent {
            body,
            to,
            from_device,
            methods
        }) if body == "@example:localhost is requesting to verify your key, ..."
            && to == user_id
            && from_device == device_id
            && methods.contains(&VerificationMethod::MSasV1)
    );
}

#[test]
#[cfg(feature = "unstable-pre-spec")]
fn verification_request_serialization() {
    let user_id = user_id!("@example2:localhost");
    let device_id: DeviceIdBox = "XOWLHHFSWM".into();
    let body = "@example:localhost is requesting to verify your key, ...".to_string();

    let methods = vec![
        VerificationMethod::MSasV1,
        VerificationMethod::_Custom("m.qr_code.show.v1".to_string()),
        VerificationMethod::_Custom("m.reciprocate.v1".to_string()),
    ];

    let json_data = json!({
        "body": body,
        "msgtype": "m.key.verification.request",
        "to": user_id,
        "from_device": device_id,
        "methods": methods
    });

    let content = MessageEventContent::VerificationRequest(KeyVerificationRequestEventContent {
        to: user_id,
        from_device: device_id,
        body,
        methods,
    });

    assert_eq!(to_json_value(&content).unwrap(), json_data,);
}

#[test]
fn content_deserialization() {
    let json_data = json!({
        "body": "test",
        "msgtype": "m.audio",
        "url": "http://example.com/audio.mp3"
    });

    assert_matches!(
        from_json_value::<Raw<MessageEventContent>>(json_data)
            .unwrap()
            .deserialize()
            .unwrap(),
        MessageEventContent::Audio(AudioMessageEventContent {
            body,
            info: None,
            url: Some(url),
            file: None,
        }) if body == "test" && url == "http://example.com/audio.mp3"
    );
}

#[test]
fn content_deserialization_failure() {
    let json_data = json!({
        "body": "test","msgtype": "m.location",
        "url": "http://example.com/audio.mp3"
    });
    assert!(from_json_value::<Raw<MessageEventContent>>(json_data).unwrap().deserialize().is_err());
}
