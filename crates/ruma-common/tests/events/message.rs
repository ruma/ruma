#![cfg(feature = "unstable-msc1767")]

use assert_matches::assert_matches;
use assign::assign;
use js_int::uint;
use ruma_common::{
    event_id,
    events::{
        emote::EmoteEventContent,
        message::{MessageContent, MessageEventContent, Text},
        notice::NoticeEventContent,
        room::message::{
            EmoteMessageEventContent, InReplyTo, MessageType, NoticeMessageEventContent, Relation,
            RoomMessageEventContent, TextMessageEventContent,
        },
        AnyMessageLikeEvent, MessageLikeEvent, MessageLikeUnsigned, OriginalMessageLikeEvent,
    },
    room_id, user_id, MilliSecondsSinceUnixEpoch,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn try_from_valid() {
    assert_matches!(
        MessageContent::try_from(vec![Text::plain("A message")]),
        Ok(message) if message.len() == 1
    );
}

#[test]
fn try_from_invalid() {
    assert_matches!(MessageContent::try_from(vec![]), Err(_));
}

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
    let event = OriginalMessageLikeEvent {
        content: MessageEventContent::plain("Hello, World!"),
        event_id: event_id!("$event:notareal.hs").to_owned(),
        sender: user_id!("@user:notareal.hs").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(134_829_848)),
        room_id: room_id!("!roomid:notareal.hs").to_owned(),
        unsigned: MessageLikeUnsigned::default(),
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
fn plain_text_content_unstable_deserialization() {
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
fn plain_text_content_stable_deserialization() {
    let json_data = json!({
        "m.text": "This is my body",
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
fn html_text_content_unstable_deserialization() {
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
fn html_text_content_stable_deserialization() {
    let json_data = json!({
        "m.message": [
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
        AnyMessageLikeEvent::Message(MessageLikeEvent::Original(OriginalMessageLikeEvent {
            content: MessageEventContent {
                message,
                ..
            },
            event_id,
            origin_server_ts,
            room_id,
            sender,
            unsigned
        })) if event_id == event_id!("$event:notareal.hs")
            && message.find_plain() == Some("Hello, World!")
            && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(134_829_848))
            && room_id == room_id!("!roomid:notareal.hs")
            && sender == user_id!("@user:notareal.hs")
            && unsigned.is_empty()
    );
}

#[test]
fn room_message_plain_text_stable_deserialization() {
    let json_data = json!({
        "body": "test",
        "msgtype": "m.text",
        "m.text": "test",
    });

    assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data)
            .unwrap(),
        RoomMessageEventContent {
            msgtype: MessageType::Text(TextMessageEventContent {
                body,
                formatted: None,
                message: Some(message),
                ..
            }),
            ..
        } if body == "test"
          && message.len() == 1
          && message[0].body == "test"
    );
}

#[test]
fn room_message_plain_text_unstable_deserialization() {
    let json_data = json!({
        "body": "test",
        "msgtype": "m.text",
        "org.matrix.msc1767.text": "test",
    });

    assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data)
            .unwrap(),
        RoomMessageEventContent {
            msgtype: MessageType::Text(TextMessageEventContent {
                body,
                formatted: None,
                message: Some(message),
                ..
            }),
            ..
        } if body == "test"
          && message.len() == 1
          && message[0].body == "test"
    );
}

#[test]
fn room_message_html_text_stable_deserialization() {
    let json_data = json!({
        "body": "test",
        "formatted_body": "<h1>test</h1>",
        "format": "org.matrix.custom.html",
        "msgtype": "m.text",
        "m.message": [
            { "body": "<h1>test</h1>", "mimetype": "text/html" },
            { "body": "test", "mimetype": "text/plain" },
        ],
    });

    assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data)
            .unwrap(),
        RoomMessageEventContent {
            msgtype: MessageType::Text(TextMessageEventContent {
                body,
                formatted: Some(formatted),
                message: Some(message),
                ..
            }),
            ..
        } if body == "test"
            && formatted.body == "<h1>test</h1>"
            && message.len() == 2
            && message[0].body == "<h1>test</h1>"
            && message[1].body == "test"
    );
}

#[test]
fn room_message_html_text_unstable_deserialization() {
    let json_data = json!({
        "body": "test",
        "formatted_body": "<h1>test</h1>",
        "format": "org.matrix.custom.html",
        "msgtype": "m.text",
        "org.matrix.msc1767.message": [
            { "body": "<h1>test</h1>", "mimetype": "text/html" },
            { "body": "test", "mimetype": "text/plain" },
        ],
    });

    assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data)
            .unwrap(),
        RoomMessageEventContent {
            msgtype: MessageType::Text(TextMessageEventContent {
                body,
                formatted: Some(formatted),
                message: Some(message),
                ..
            }),
            ..
        } if body == "test"
            && formatted.body == "<h1>test</h1>"
            && message.len() == 2
            && message[0].body == "<h1>test</h1>"
            && message[1].body == "test"
    );
}

#[test]
fn notice_event_serialization() {
    let event = OriginalMessageLikeEvent {
        content: NoticeEventContent::plain("Hello, I'm a robot!"),
        event_id: event_id!("$event:notareal.hs").to_owned(),
        sender: user_id!("@user:notareal.hs").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(134_829_848)),
        room_id: room_id!("!roomid:notareal.hs").to_owned(),
        unsigned: MessageLikeUnsigned::default(),
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
fn room_message_notice_serialization() {
    let message_event_content =
        RoomMessageEventContent::notice_plain("> <@test:example.com> test\n\ntest reply");

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "> <@test:example.com> test\n\ntest reply",
            "msgtype": "m.notice",
            "org.matrix.msc1767.text": "> <@test:example.com> test\n\ntest reply",
        })
    );
}

#[test]
fn notice_event_stable_deserialization() {
    let json_data = json!({
        "content": {
            "m.message": [
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
        AnyMessageLikeEvent::Notice(MessageLikeEvent::Original(OriginalMessageLikeEvent {
            content: NoticeEventContent {
                message,
                ..
            },
            event_id,
            origin_server_ts,
            room_id,
            sender,
            unsigned
        })) if event_id == event_id!("$event:notareal.hs")
            && message.find_plain() == Some("Hello, I'm a robot!")
            && message.find_html() == Some("Hello, I'm a <em>robot</em>!")
            && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(134_829_848))
            && room_id == room_id!("!roomid:notareal.hs")
            && sender == user_id!("@user:notareal.hs")
            && unsigned.is_empty()
    );
}

#[test]
fn notice_event_unstable_deserialization() {
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
        AnyMessageLikeEvent::Notice(MessageLikeEvent::Original(OriginalMessageLikeEvent {
            content: NoticeEventContent {
                message,
                ..
            },
            event_id,
            origin_server_ts,
            room_id,
            sender,
            unsigned
        })) if event_id == event_id!("$event:notareal.hs")
            && message.find_plain() == Some("Hello, I'm a robot!")
            && message.find_html() == Some("Hello, I'm a <em>robot</em>!")
            && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(134_829_848))
            && room_id == room_id!("!roomid:notareal.hs")
            && sender == user_id!("@user:notareal.hs")
            && unsigned.is_empty()
    );
}

#[test]
fn room_message_notice_stable_deserialization() {
    let json_data = json!({
        "body": "test",
        "msgtype": "m.notice",
        "m.text": "test",
    });

    assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data)
            .unwrap(),
        RoomMessageEventContent {
            msgtype: MessageType::Notice(NoticeMessageEventContent {
                body,
                formatted: None,
                message: Some(message),
                ..
            }),
            ..
        } if body == "test"
          && message.len() == 1
          && message[0].body == "test"
    );
}

#[test]
fn room_message_notice_unstable_deserialization() {
    let json_data = json!({
        "body": "test",
        "msgtype": "m.notice",
        "org.matrix.msc1767.text": "test",
    });

    assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data)
            .unwrap(),
        RoomMessageEventContent {
            msgtype: MessageType::Notice(NoticeMessageEventContent {
                body,
                formatted: None,
                message: Some(message),
                ..
            }),
            ..
        } if body == "test"
          && message.len() == 1
          && message[0].body == "test"
    );
}

#[test]
fn emote_event_serialization() {
    let event = OriginalMessageLikeEvent {
        content: EmoteEventContent::html(
            "is testing some code…",
            "is testing some <code>code</code>…",
        ),
        event_id: event_id!("$event:notareal.hs").to_owned(),
        sender: user_id!("@user:notareal.hs").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(134_829_848)),
        room_id: room_id!("!roomid:notareal.hs").to_owned(),
        unsigned: MessageLikeUnsigned::default(),
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
fn room_message_emote_serialization() {
    let message_event_content = RoomMessageEventContent::new(MessageType::Emote(
        EmoteMessageEventContent::plain("> <@test:example.com> test\n\ntest reply"),
    ));

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "> <@test:example.com> test\n\ntest reply",
            "msgtype": "m.emote",
            "org.matrix.msc1767.text": "> <@test:example.com> test\n\ntest reply",
        })
    );
}

#[test]
fn emote_event_stable_deserialization() {
    let json_data = json!({
        "content": {
            "m.text": "is testing some code…",
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.emote",
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::Emote(MessageLikeEvent::Original(OriginalMessageLikeEvent {
            content: EmoteEventContent {
                message,
                ..
            },
            event_id,
            origin_server_ts,
            room_id,
            sender,
            unsigned
        })) if event_id == event_id!("$event:notareal.hs")
            && message.find_plain() == Some("is testing some code…")
            && message.find_html().is_none()
            && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(134_829_848))
            && room_id == room_id!("!roomid:notareal.hs")
            && sender == user_id!("@user:notareal.hs")
            && unsigned.is_empty()
    );
}

#[test]
fn emote_event_unstable_deserialization() {
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
        AnyMessageLikeEvent::Emote(MessageLikeEvent::Original(OriginalMessageLikeEvent {
            content: EmoteEventContent {
                message,
                ..
            },
            event_id,
            origin_server_ts,
            room_id,
            sender,
            unsigned
        })) if event_id == event_id!("$event:notareal.hs")
            && message.find_plain() == Some("is testing some code…")
            && message.find_html().is_none()
            && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(134_829_848))
            && room_id == room_id!("!roomid:notareal.hs")
            && sender == user_id!("@user:notareal.hs")
            && unsigned.is_empty()
    );
}

#[test]
fn room_message_emote_stable_deserialization() {
    let json_data = json!({
        "body": "test",
        "msgtype": "m.emote",
        "m.text": "test",
    });

    assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data)
            .unwrap(),
        RoomMessageEventContent {
            msgtype: MessageType::Emote(EmoteMessageEventContent {
                body,
                formatted: None,
                message: Some(message),
                ..
            }),
            ..
        } if body == "test"
          && message.len() == 1
          && message[0].body == "test"
    );
}

#[test]
fn room_message_emote_unstable_deserialization() {
    let json_data = json!({
        "body": "test",
        "msgtype": "m.emote",
        "org.matrix.msc1767.text": "test",
    });

    assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data)
            .unwrap(),
        RoomMessageEventContent {
            msgtype: MessageType::Emote(EmoteMessageEventContent {
                body,
                formatted: None,
                message: Some(message),
                ..
            }),
            ..
        } if body == "test"
          && message.len() == 1
          && message[0].body == "test"
    );
}

#[test]
#[cfg(feature = "unstable-msc3554")]
fn lang_serialization() {
    let content = MessageContent::try_from(vec![
        assign!(Text::plain("Bonjour le monde !"), { lang: Some("fr".into()) }),
        assign!(Text::plain("Hallo Welt!"), { lang: Some("de".into()) }),
        assign!(Text::plain("Hello World!"), { lang: Some("en".into()) }),
    ])
    .unwrap();

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "org.matrix.msc1767.message": [
                { "body": "Bonjour le monde !", "mimetype": "text/plain", "lang": "fr"},
                { "body": "Hallo Welt!", "mimetype": "text/plain", "lang": "de"},
                { "body": "Hello World!", "mimetype": "text/plain", "lang": "en"},
            ]
        })
    );
}

#[test]
#[cfg(feature = "unstable-msc3554")]
fn lang_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.message": [
            { "body": "Bonjour le monde !", "mimetype": "text/plain", "lang": "fr"},
            { "body": "Hallo Welt!", "mimetype": "text/plain", "lang": "de"},
            { "body": "Hello World!", "mimetype": "text/plain", "lang": "en"},
        ]
    });

    let content = from_json_value::<MessageContent>(json_data).unwrap();
    assert_eq!(content[0].lang.as_deref(), Some("fr"));
    assert_eq!(content[1].lang.as_deref(), Some("de"));
    assert_eq!(content[2].lang.as_deref(), Some("en"));
}
