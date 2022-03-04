#![cfg(feature = "unstable-msc1767")]

use assign::assign;
use js_int::uint;
use matches::assert_matches;
use ruma_common::{
    event_id,
    events::{
        emote::EmoteEventContent,
        message::MessageEventContent,
        notice::NoticeEventContent,
        room::message::{InReplyTo, Relation},
        AnyMessageLikeEvent, MessageLikeEvent, Unsigned,
    },
    room_id, user_id, MilliSecondsSinceUnixEpoch,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn html_content_serialization() {
    let message_event_content =
        MessageEventContent::html("Hello, World!", "Hello, <em>World</em>!");

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "org.matrix.msc1767.message": [
                { "body": "Hello, <em>World</em>!", "mimetype": "text/html"},
                { "body": "Hello, World!", "mimetype": "text/plain"},
            ]
        })
    );
}

#[test]
fn plain_text_content_serialization() {
    let message_event_content =
        MessageEventContent::plain("> <@test:example.com> test\n\ntest reply");

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "org.matrix.msc1767.text": "> <@test:example.com> test\n\ntest reply",
        })
    );
}

#[test]
#[cfg(feature = "markdown")]
fn markdown_content_serialization() {
    let formatted_message = MessageEventContent::markdown("Testing **bold** and _italic_!");

    assert_eq!(
        to_json_value(&formatted_message).unwrap(),
        json!({
            "org.matrix.msc1767.message": [
                { "body": "<p>Testing <strong>bold</strong> and <em>italic</em>!</p>\n", "mimetype": "text/html"},
                { "body": "Testing **bold** and _italic_!", "mimetype": "text/plain"},
            ]
        })
    );

    let plain_message_simple = MessageEventContent::markdown("Testing a simple phrase…");

    assert_eq!(
        to_json_value(&plain_message_simple).unwrap(),
        json!({
            "org.matrix.msc1767.text": "Testing a simple phrase…",
        })
    );

    let plain_message_paragraphs =
        MessageEventContent::markdown("Testing\n\nSeveral\n\nParagraphs.");

    assert_eq!(
        to_json_value(&plain_message_paragraphs).unwrap(),
        json!({
            "org.matrix.msc1767.message": [
                { "body": "<p>Testing</p>\n<p>Several</p>\n<p>Paragraphs.</p>\n", "mimetype": "text/html"},
                { "body": "Testing\n\nSeveral\n\nParagraphs.", "mimetype": "text/plain"},
            ]
        })
    );
}

#[test]
fn relates_to_content_serialization() {
    let message_event_content =
        assign!(MessageEventContent::plain("> <@test:example.com> test\n\ntest reply"), {
            relates_to: Some(Relation::Reply {
                in_reply_to: InReplyTo::new(
                    event_id!("$15827405538098VGFWH:example.com").to_owned(),
                ),
            }),
        });

    let json_data = json!({
        "org.matrix.msc1767.text": "> <@test:example.com> test\n\ntest reply",
        "m.relates_to": {
            "m.in_reply_to": {
                "event_id": "$15827405538098VGFWH:example.com"
            }
        }
    });

    assert_eq!(to_json_value(&message_event_content).unwrap(), json_data);
}

#[test]
fn message_event_serialization() {
    let event = MessageLikeEvent {
        content: MessageEventContent::plain("Hello, World!"),
        event_id: event_id!("$event:notareal.hs").to_owned(),
        sender: user_id!("@user:notareal.hs").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(134_829_848)),
        room_id: room_id!("!roomid:notareal.hs").to_owned(),
        unsigned: Unsigned::default(),
    };

    assert_eq!(
        to_json_value(&event).unwrap(),
        json!({
            "content": {
                "org.matrix.msc1767.text": "Hello, World!",
            },
            "event_id": "$event:notareal.hs",
            "origin_server_ts": 134_829_848,
            "room_id": "!roomid:notareal.hs",
            "sender": "@user:notareal.hs",
            "type": "m.message",
        })
    );
}

#[test]
fn plain_text_content_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.text": "This is my body",
    });

    assert_matches!(
        from_json_value::<MessageEventContent>(json_data)
            .unwrap(),
        MessageEventContent { message, .. }
        if message.find_plain() == Some("This is my body")
            && message.find_html().is_none()
    );
}

#[test]
fn html_text_content_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.message": [
            { "body": "Hello, <em>New World</em>!", "mimetype": "text/html"},
            { "body": "Hello, New World!" },
        ]
    });

    assert_matches!(
        from_json_value::<MessageEventContent>(json_data)
            .unwrap(),
        MessageEventContent { message, .. }
        if message.find_plain() == Some("Hello, New World!")
            && message.find_html() == Some("Hello, <em>New World</em>!")
    );
}

#[test]
fn relates_to_content_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.text": "> <@test:example.com> test\n\ntest reply",
        "m.relates_to": {
            "m.in_reply_to": {
                "event_id": "$15827405538098VGFWH:example.com"
            }
        }
    });

    assert_matches!(
        from_json_value::<MessageEventContent>(json_data)
            .unwrap(),
        MessageEventContent {
            message,
            relates_to: Some(Relation::Reply { in_reply_to: InReplyTo { event_id, .. } }),
            ..
        }
        if message.find_plain() == Some("> <@test:example.com> test\n\ntest reply")
            && message.find_html().is_none()
            && event_id == event_id!("$15827405538098VGFWH:example.com")
    );
}

#[test]
fn message_event_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc1767.text": "Hello, World!",
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.message",
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::Message(MessageLikeEvent {
            content: MessageEventContent {
                message,
                ..
            },
            event_id,
            origin_server_ts,
            room_id,
            sender,
            unsigned
        }) if event_id == event_id!("$event:notareal.hs")
            && message.find_plain() == Some("Hello, World!")
            && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(134_829_848))
            && room_id == room_id!("!roomid:notareal.hs")
            && sender == user_id!("@user:notareal.hs")
            && unsigned.is_empty()
    );
}

#[test]
fn notice_event_serialization() {
    let event = MessageLikeEvent {
        content: NoticeEventContent::plain("Hello, I'm a robot!"),
        event_id: event_id!("$event:notareal.hs").to_owned(),
        sender: user_id!("@user:notareal.hs").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(134_829_848)),
        room_id: room_id!("!roomid:notareal.hs").to_owned(),
        unsigned: Unsigned::default(),
    };

    assert_eq!(
        to_json_value(&event).unwrap(),
        json!({
            "content": {
                "org.matrix.msc1767.text": "Hello, I'm a robot!",
            },
            "event_id": "$event:notareal.hs",
            "origin_server_ts": 134_829_848,
            "room_id": "!roomid:notareal.hs",
            "sender": "@user:notareal.hs",
            "type": "m.notice",
        })
    );
}

#[test]
fn notice_event_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc1767.message": [
                { "body": "Hello, I'm a <em>robot</em>!", "mimetype": "text/html"},
                { "body": "Hello, I'm a robot!" },
            ]
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.notice",
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::Notice(MessageLikeEvent {
            content: NoticeEventContent {
                message,
                ..
            },
            event_id,
            origin_server_ts,
            room_id,
            sender,
            unsigned
        }) if event_id == event_id!("$event:notareal.hs")
            && message.find_plain() == Some("Hello, I'm a robot!")
            && message.find_html() == Some("Hello, I'm a <em>robot</em>!")
            && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(134_829_848))
            && room_id == room_id!("!roomid:notareal.hs")
            && sender == user_id!("@user:notareal.hs")
            && unsigned.is_empty()
    );
}

#[test]
fn emote_event_serialization() {
    let event = MessageLikeEvent {
        content: EmoteEventContent::html(
            "is testing some code…",
            "is testing some <code>code</code>…",
        ),
        event_id: event_id!("$event:notareal.hs").to_owned(),
        sender: user_id!("@user:notareal.hs").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(134_829_848)),
        room_id: room_id!("!roomid:notareal.hs").to_owned(),
        unsigned: Unsigned::default(),
    };

    assert_eq!(
        to_json_value(&event).unwrap(),
        json!({
            "content": {
                "org.matrix.msc1767.message": [
                    { "body": "is testing some <code>code</code>…", "mimetype": "text/html" },
                    { "body": "is testing some code…", "mimetype": "text/plain" },
                ]
            },
            "event_id": "$event:notareal.hs",
            "origin_server_ts": 134_829_848,
            "room_id": "!roomid:notareal.hs",
            "sender": "@user:notareal.hs",
            "type": "m.emote",
        })
    );
}

#[test]
fn emote_event_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc1767.text": "is testing some code…",
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.emote",
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::Emote(MessageLikeEvent {
            content: EmoteEventContent {
                message,
                ..
            },
            event_id,
            origin_server_ts,
            room_id,
            sender,
            unsigned
        }) if event_id == event_id!("$event:notareal.hs")
            && message.find_plain() == Some("is testing some code…")
            && message.find_html().is_none()
            && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(134_829_848))
            && room_id == room_id!("!roomid:notareal.hs")
            && sender == user_id!("@user:notareal.hs")
            && unsigned.is_empty()
    );
}
