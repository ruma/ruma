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
        relation::InReplyTo,
        room::message::{EmoteMessageEventContent, MessageType, Relation, RoomMessageEventContent},
        AnyMessageLikeEvent, MessageLikeEvent,
    },
    serde::CanBeEmpty,
    MilliSecondsSinceUnixEpoch,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn try_from_valid() {
    let message = MessageContent::try_from(vec![Text::plain("A message")]).unwrap();
    assert_eq!(message.len(), 1);
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
            "org.matrix.msc1767.html": "Hello, <em>World</em>!",
            "org.matrix.msc1767.text": "Hello, World!",
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
fn unknown_mimetype_content_serialization() {
    let message_event_content = MessageEventContent::from(
        MessageContent::try_from(vec![
            Text::plain("> <@test:example.com> test\n\ntest reply"),
            Text::new(
                "application/json",
                r#"{ "quote": "<@test:example.com> test", "reply": "test reply" }"#,
            ),
        ])
        .unwrap(),
    );

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "org.matrix.msc1767.message": [
                {
                    "body": "> <@test:example.com> test\n\ntest reply",
                    "mimetype": "text/plain",
                },
                {
                    "body": r#"{ "quote": "<@test:example.com> test", "reply": "test reply" }"#,
                    "mimetype": "application/json",
                },
            ]
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
            "org.matrix.msc1767.html": "<p>Testing <strong>bold</strong> and <em>italic</em>!</p>\n",
            "org.matrix.msc1767.text": "Testing **bold** and _italic_!",
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
            "org.matrix.msc1767.html": "<p>Testing</p>\n<p>Several</p>\n<p>Paragraphs.</p>\n",
            "org.matrix.msc1767.text": "Testing\n\nSeveral\n\nParagraphs.",
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
    let content = MessageEventContent::plain("Hello, World!");
    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({ "org.matrix.msc1767.text": "Hello, World!" })
    );
}

#[test]
fn plain_text_content_unstable_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.text": "This is my body",
    });

    let content = from_json_value::<MessageEventContent>(json_data).unwrap();
    assert_eq!(content.message.find_plain(), Some("This is my body"));
    assert_eq!(content.message.find_html(), None);
}

#[test]
fn plain_text_content_stable_deserialization() {
    let json_data = json!({
        "m.text": "This is my body",
    });

    let content = from_json_value::<MessageEventContent>(json_data).unwrap();
    assert_eq!(content.message.find_plain(), Some("This is my body"));
    assert_eq!(content.message.find_html(), None);
}

#[test]
fn html_content_unstable_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.html": "Hello, <em>New World</em>!",
    });

    let content = from_json_value::<MessageEventContent>(json_data).unwrap();
    assert_eq!(content.message.find_plain(), None);
    assert_eq!(content.message.find_html(), Some("Hello, <em>New World</em>!"));
}

#[test]
fn html_content_stable_deserialization() {
    let json_data = json!({
        "m.html": "Hello, <em>New World</em>!",
    });

    let content = from_json_value::<MessageEventContent>(json_data).unwrap();
    assert_eq!(content.message.find_plain(), None);
    assert_eq!(content.message.find_html(), Some("Hello, <em>New World</em>!"));
}

#[test]
fn html_and_text_content_unstable_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.html": "Hello, <em>New World</em>!",
        "org.matrix.msc1767.text": "Hello, New World!",
    });

    let content = from_json_value::<MessageEventContent>(json_data).unwrap();
    assert_eq!(content.message.find_plain(), Some("Hello, New World!"));
    assert_eq!(content.message.find_html(), Some("Hello, <em>New World</em>!"));
}

#[test]
fn html_and_text_content_stable_deserialization() {
    let json_data = json!({
        "m.html": "Hello, <em>New World</em>!",
        "m.text": "Hello, New World!",
    });

    let content = from_json_value::<MessageEventContent>(json_data).unwrap();
    assert_eq!(content.message.find_plain(), Some("Hello, New World!"));
    assert_eq!(content.message.find_html(), Some("Hello, <em>New World</em>!"));
}

#[test]
fn message_content_unstable_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.message": [
            { "body": "Hello, <em>New World</em>!", "mimetype": "text/html"},
            { "body": "Hello, New World!" },
        ]
    });

    let content = from_json_value::<MessageEventContent>(json_data).unwrap();
    assert_eq!(content.message.find_plain(), Some("Hello, New World!"));
    assert_eq!(content.message.find_html(), Some("Hello, <em>New World</em>!"));
}

#[test]
fn message_content_stable_deserialization() {
    let json_data = json!({
        "m.message": [
            { "body": "Hello, <em>New World</em>!", "mimetype": "text/html"},
            { "body": "Hello, New World!" },
        ]
    });

    let content = from_json_value::<MessageEventContent>(json_data).unwrap();
    assert_eq!(content.message.find_plain(), Some("Hello, New World!"));
    assert_eq!(content.message.find_html(), Some("Hello, <em>New World</em>!"));
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

    let content = from_json_value::<MessageEventContent>(json_data).unwrap();
    assert_eq!(content.message.find_plain(), Some("> <@test:example.com> test\n\ntest reply"));
    assert_eq!(content.message.find_html(), None);

    let event_id = assert_matches!(
        content.relates_to,
        Some(Relation::Reply { in_reply_to: InReplyTo { event_id, .. } }) => event_id
    );
    assert_eq!(event_id, "$15827405538098VGFWH:example.com");
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

    let message_event = assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data),
        Ok(AnyMessageLikeEvent::Message(MessageLikeEvent::Original(message_event))) => message_event
    );
    assert_eq!(message_event.event_id, "$event:notareal.hs");
    assert_eq!(message_event.content.message.find_plain(), Some("Hello, World!"));
    assert_eq!(message_event.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(134_829_848)));
    assert_eq!(message_event.room_id, "!roomid:notareal.hs");
    assert_eq!(message_event.sender, "@user:notareal.hs");
    assert!(message_event.unsigned.is_empty());
}

#[test]
fn room_message_plain_text_stable_deserialization() {
    let json_data = json!({
        "body": "test",
        "msgtype": "m.text",
        "m.text": "test",
    });

    let content = assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data),
        Ok(RoomMessageEventContent {
            msgtype: MessageType::Text(content),
            ..
        }) => content
    );
    assert_eq!(content.body, "test");
    let message = content.message.unwrap();
    assert_eq!(message.len(), 1);
    assert_eq!(message[0].body, "test");
}

#[test]
fn room_message_plain_text_unstable_deserialization() {
    let json_data = json!({
        "body": "test",
        "msgtype": "m.text",
        "org.matrix.msc1767.text": "test",
    });

    let content = assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data),
        Ok(RoomMessageEventContent {
            msgtype: MessageType::Text(content),
            ..
        }) => content
    );
    assert_eq!(content.body, "test");
    let message = content.message.unwrap();
    assert_eq!(message.len(), 1);
    assert_eq!(message[0].body, "test");
}

#[test]
fn room_message_html_and_text_stable_deserialization() {
    let json_data = json!({
        "body": "test",
        "formatted_body": "<h1>test</h1>",
        "format": "org.matrix.custom.html",
        "msgtype": "m.text",
        "m.html": "<h1>test</h1>",
        "m.text": "test",
    });

    let content = assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data),
        Ok(RoomMessageEventContent {
            msgtype: MessageType::Text(content),
            ..
        }) => content
    );
    assert_eq!(content.body, "test");
    let formatted = content.formatted.unwrap();
    assert_eq!(formatted.body, "<h1>test</h1>");
    let message = content.message.unwrap();
    assert_eq!(message.len(), 2);
    assert_eq!(message[0].body, "<h1>test</h1>");
    assert_eq!(message[1].body, "test");
}

#[test]
fn room_message_html_and_text_unstable_deserialization() {
    let json_data = json!({
        "body": "test",
        "formatted_body": "<h1>test</h1>",
        "format": "org.matrix.custom.html",
        "msgtype": "m.text",
        "org.matrix.msc1767.html": "<h1>test</h1>",
        "org.matrix.msc1767.text": "test",
    });

    let content = assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data),
        Ok(RoomMessageEventContent {
            msgtype: MessageType::Text(content),
            ..
        }) => content
    );
    assert_eq!(content.body, "test");
    let formatted = content.formatted.unwrap();
    assert_eq!(formatted.body, "<h1>test</h1>");
    let message = content.message.unwrap();
    assert_eq!(message.len(), 2);
    assert_eq!(message[0].body, "<h1>test</h1>");
    assert_eq!(message[1].body, "test");
}

#[test]
fn room_message_message_stable_deserialization() {
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

    let content = assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data),
        Ok(RoomMessageEventContent {
            msgtype: MessageType::Text(content),
            ..
        }) => content
    );
    assert_eq!(content.body, "test");
    let formatted = content.formatted.unwrap();
    assert_eq!(formatted.body, "<h1>test</h1>");
    let message = content.message.unwrap();
    assert_eq!(message.len(), 2);
    assert_eq!(message[0].body, "<h1>test</h1>");
    assert_eq!(message[1].body, "test");
}

#[test]
fn room_message_message_unstable_deserialization() {
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

    let content = assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data),
        Ok(RoomMessageEventContent {
            msgtype: MessageType::Text(content),
            ..
        }) => content
    );
    assert_eq!(content.body, "test");
    let formatted = content.formatted.unwrap();
    assert_eq!(formatted.body, "<h1>test</h1>");
    let message = content.message.unwrap();
    assert_eq!(message.len(), 2);
    assert_eq!(message[0].body, "<h1>test</h1>");
    assert_eq!(message[1].body, "test");
}

#[test]
fn notice_event_serialization() {
    let content = NoticeEventContent::plain("Hello, I'm a robot!");
    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({ "org.matrix.msc1767.text": "Hello, I'm a robot!" })
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

    let message_event = assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data),
        Ok(AnyMessageLikeEvent::Notice(MessageLikeEvent::Original(message_event))) => message_event
    );

    assert_eq!(message_event.event_id, "$event:notareal.hs");
    assert_eq!(message_event.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(134_829_848)));
    assert_eq!(message_event.room_id, "!roomid:notareal.hs");
    assert_eq!(message_event.sender, "@user:notareal.hs");
    assert!(message_event.unsigned.is_empty());

    let message = message_event.content.message;
    assert_eq!(message.find_plain(), Some("Hello, I'm a robot!"));
    assert_eq!(message.find_html(), Some("Hello, I'm a <em>robot</em>!"));
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

    let message_event = assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data),
        Ok(AnyMessageLikeEvent::Notice(MessageLikeEvent::Original(message_event))) => message_event
    );

    assert_eq!(message_event.event_id, "$event:notareal.hs");
    assert_eq!(message_event.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(134_829_848)));
    assert_eq!(message_event.room_id, "!roomid:notareal.hs");
    assert_eq!(message_event.sender, "@user:notareal.hs");
    assert!(message_event.unsigned.is_empty());

    let message = message_event.content.message;
    assert_eq!(message.find_plain(), Some("Hello, I'm a robot!"));
    assert_eq!(message.find_html(), Some("Hello, I'm a <em>robot</em>!"));
}

#[test]
fn room_message_notice_stable_deserialization() {
    let json_data = json!({
        "body": "test",
        "msgtype": "m.notice",
        "m.text": "test",
    });

    let content = assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data),
        Ok(RoomMessageEventContent {
            msgtype: MessageType::Notice(content),
            ..
        }) => content
    );
    assert_eq!(content.body, "test");
    let message = content.message.unwrap();
    assert_eq!(message.len(), 1);
    assert_eq!(message[0].body, "test");
}

#[test]
fn room_message_notice_unstable_deserialization() {
    let json_data = json!({
        "body": "test",
        "msgtype": "m.notice",
        "org.matrix.msc1767.text": "test",
    });

    let content = assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data),
        Ok(RoomMessageEventContent {
            msgtype: MessageType::Notice(content),
            ..
        }) => content
    );
    assert_eq!(content.body, "test");
    let message = content.message.unwrap();
    assert_eq!(message.len(), 1);
    assert_eq!(message[0].body, "test");
}

#[test]
fn emote_event_serialization() {
    let content =
        EmoteEventContent::html("is testing some code…", "is testing some <code>code</code>…");

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "org.matrix.msc1767.html": "is testing some <code>code</code>…",
            "org.matrix.msc1767.text": "is testing some code…",
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

    let message_event = assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data),
        Ok(AnyMessageLikeEvent::Emote(MessageLikeEvent::Original(message_event))) => message_event
    );

    assert_eq!(message_event.event_id, "$event:notareal.hs");
    assert_eq!(message_event.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(134_829_848)));
    assert_eq!(message_event.room_id, "!roomid:notareal.hs");
    assert_eq!(message_event.sender, "@user:notareal.hs");
    assert!(message_event.unsigned.is_empty());

    let message = message_event.content.message;
    assert_eq!(message.find_plain(), Some("is testing some code…"));
    assert_eq!(message.find_html(), None);
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

    let message_event = assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data),
        Ok(AnyMessageLikeEvent::Emote(MessageLikeEvent::Original(message_event))) => message_event
    );

    assert_eq!(message_event.event_id, "$event:notareal.hs");
    assert_eq!(message_event.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(134_829_848)));
    assert_eq!(message_event.room_id, "!roomid:notareal.hs");
    assert_eq!(message_event.sender, "@user:notareal.hs");
    assert!(message_event.unsigned.is_empty());

    let message = message_event.content.message;
    assert_eq!(message.find_plain(), Some("is testing some code…"));
    assert_eq!(message.find_html(), None);
}

#[test]
fn room_message_emote_stable_deserialization() {
    let json_data = json!({
        "body": "test",
        "msgtype": "m.emote",
        "m.text": "test",
    });

    let content = assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data),
        Ok(RoomMessageEventContent {
            msgtype: MessageType::Emote(content),
            ..
        }) => content
    );
    assert_eq!(content.body, "test");
    let message = content.message.unwrap();
    assert_eq!(message.len(), 1);
    assert_eq!(message[0].body, "test");
}

#[test]
fn room_message_emote_unstable_deserialization() {
    let json_data = json!({
        "body": "test",
        "msgtype": "m.emote",
        "org.matrix.msc1767.text": "test",
    });

    let content = assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data),
        Ok(RoomMessageEventContent {
            msgtype: MessageType::Emote(content),
            ..
        }) => content
    );
    assert_eq!(content.body, "test");
    let message = content.message.unwrap();
    assert_eq!(message.len(), 1);
    assert_eq!(message[0].body, "test");
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
        to_json_value(content).unwrap(),
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
