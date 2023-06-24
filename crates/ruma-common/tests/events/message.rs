#![cfg(feature = "unstable-msc1767")]

use assert_matches2::assert_matches;
use assign::assign;
use js_int::uint;
#[cfg(feature = "unstable-msc3954")]
use ruma_common::events::emote::EmoteEventContent;
use ruma_common::{
    events::{
        message::{MessageEventContent, TextContentBlock, TextRepresentation},
        relation::InReplyTo,
        room::message::Relation,
        AnyMessageLikeEvent, MessageLikeEvent,
    },
    owned_event_id,
    serde::CanBeEmpty,
    MilliSecondsSinceUnixEpoch,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn html_content_serialization() {
    let message_event_content =
        MessageEventContent::html("Hello, World!", "Hello, <em>World</em>!");

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                { "mimetype": "text/html", "body": "Hello, <em>World</em>!" },
                { "body": "Hello, World!" },
            ],
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
            "org.matrix.msc1767.text": [
                { "body": "> <@test:example.com> test\n\ntest reply" },
            ],
        })
    );
}

#[test]
fn unknown_mimetype_content_serialization() {
    let message_event_content = MessageEventContent::from(
        TextContentBlock::try_from(vec![
            TextRepresentation::plain("> <@test:example.com> test\n\ntest reply"),
            TextRepresentation::new(
                "application/json",
                r#"{ "quote": "<@test:example.com> test", "reply": "test reply" }"#,
            ),
        ])
        .unwrap(),
    );

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                {
                    "body": "> <@test:example.com> test\n\ntest reply",
                },
                {
                    "body": r#"{ "quote": "<@test:example.com> test", "reply": "test reply" }"#,
                    "mimetype": "application/json",
                },
            ],
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
            "org.matrix.msc1767.text": [
                {
                    "mimetype": "text/html",
                    "body": "<p>Testing <strong>bold</strong> and <em>italic</em>!</p>\n",
                },
                {
                    "body": "Testing **bold** and _italic_!",
                },
            ],
        })
    );

    let plain_message_simple = MessageEventContent::markdown("Testing a simple phrase…");

    assert_eq!(
        to_json_value(&plain_message_simple).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                { "body": "Testing a simple phrase…" },
            ],
        })
    );

    let plain_message_paragraphs =
        MessageEventContent::markdown("Testing\n\nSeveral\n\nParagraphs.");

    assert_eq!(
        to_json_value(&plain_message_paragraphs).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                {
                    "mimetype": "text/html",
                    "body": "<p>Testing</p>\n<p>Several</p>\n<p>Paragraphs.</p>\n",
                },
                {
                    "body": "Testing\n\nSeveral\n\nParagraphs.",
                },
            ],
        })
    );
}

#[test]
fn reply_content_serialization() {
    #[rustfmt::skip] // rustfmt wants to merge the next two lines
    let message_event_content =
        assign!(MessageEventContent::plain("> <@test:example.com> test\n\ntest reply"), {
            relates_to: Some(Relation::Reply {
                in_reply_to: InReplyTo::new(
                    owned_event_id!("$15827405538098VGFWH:example.com"),
                ),
            }),
        });

    let json_data = json!({
        "org.matrix.msc1767.text": [
            { "body": "> <@test:example.com> test\n\ntest reply" },
        ],
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
        json!({
            "org.matrix.msc1767.text": [
                { "body": "Hello, World!" },
            ],
        })
    );
}

#[test]
fn plain_text_content_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.text": [
            { "body": "This is my body" },
        ],
    });

    let content = from_json_value::<MessageEventContent>(json_data).unwrap();
    assert_eq!(content.text.find_plain(), Some("This is my body"));
    assert_eq!(content.text.find_html(), None);
    #[cfg(feature = "unstable-msc3955")]
    assert!(!content.automated);
}

#[test]
fn html_content_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.text": [
            { "mimetype": "text/html", "body": "Hello, <em>New World</em>!" },
        ]
    });

    let content = from_json_value::<MessageEventContent>(json_data).unwrap();
    assert_eq!(content.text.find_plain(), None);
    assert_eq!(content.text.find_html(), Some("Hello, <em>New World</em>!"));
    #[cfg(feature = "unstable-msc3955")]
    assert!(!content.automated);
}

#[test]
fn html_and_text_content_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.text": [
            { "mimetype": "text/html", "body": "Hello, <em>New World</em>!" },
            { "body": "Hello, New World!" },
        ],
    });

    let content = from_json_value::<MessageEventContent>(json_data).unwrap();
    assert_eq!(content.text.find_plain(), Some("Hello, New World!"));
    assert_eq!(content.text.find_html(), Some("Hello, <em>New World</em>!"));
    #[cfg(feature = "unstable-msc3955")]
    assert!(!content.automated);
}

#[test]
fn reply_content_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.text": [
            { "body": "> <@test:example.com> test\n\ntest reply" },
        ],
        "m.relates_to": {
            "m.in_reply_to": {
                "event_id": "$15827405538098VGFWH:example.com"
            }
        }
    });

    let content = from_json_value::<MessageEventContent>(json_data).unwrap();
    assert_eq!(content.text.find_plain(), Some("> <@test:example.com> test\n\ntest reply"));
    assert_eq!(content.text.find_html(), None);

    assert_matches!(
        content.relates_to,
        Some(Relation::Reply { in_reply_to: InReplyTo { event_id, .. } })
    );
    assert_eq!(event_id, "$15827405538098VGFWH:example.com");
}

#[test]
fn thread_content_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.text": [
            { "body": "Test in thread" },
        ],
        "m.relates_to": {
            "rel_type": "m.thread",
            "event_id": "$15827405538098VGFWH:example.com",
        }
    });

    let content = from_json_value::<MessageEventContent>(json_data).unwrap();
    assert_eq!(content.text.find_plain(), Some("Test in thread"));
    assert_eq!(content.text.find_html(), None);

    assert_matches!(content.relates_to, Some(Relation::Thread(thread)));
    assert_eq!(thread.event_id, "$15827405538098VGFWH:example.com");
}

#[test]
fn message_event_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc1767.text": [
                { "body": "Hello, World!" },
            ],
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "org.matrix.msc1767.message",
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data),
        Ok(AnyMessageLikeEvent::Message(MessageLikeEvent::Original(message_event)))
    );
    assert_eq!(message_event.event_id, "$event:notareal.hs");
    assert_eq!(message_event.content.text.find_plain(), Some("Hello, World!"));
    assert_eq!(message_event.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(134_829_848)));
    assert_eq!(message_event.room_id, "!roomid:notareal.hs");
    assert_eq!(message_event.sender, "@user:notareal.hs");
    assert!(message_event.unsigned.is_empty());
}

#[test]
#[cfg(feature = "unstable-msc3954")]
fn emote_event_serialization() {
    let content =
        EmoteEventContent::html("is testing some code…", "is testing some <code>code</code>…");

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                { "mimetype": "text/html", "body": "is testing some <code>code</code>…" },
                { "body": "is testing some code…" },
            ],
        })
    );
}

#[test]
#[cfg(feature = "unstable-msc3954")]
fn emote_event_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc1767.text": [
                { "body": "is testing some code…" },
            ],
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "org.matrix.msc1767.emote",
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data),
        Ok(AnyMessageLikeEvent::Emote(MessageLikeEvent::Original(message_event)))
    );

    assert_eq!(message_event.event_id, "$event:notareal.hs");
    assert_eq!(message_event.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(134_829_848)));
    assert_eq!(message_event.room_id, "!roomid:notareal.hs");
    assert_eq!(message_event.sender, "@user:notareal.hs");
    assert!(message_event.unsigned.is_empty());

    let text = message_event.content.text;
    assert_eq!(text.find_plain(), Some("is testing some code…"));
    assert_eq!(text.find_html(), None);
}

#[test]
#[cfg(feature = "unstable-msc3554")]
fn lang_serialization() {
    let content = TextContentBlock::try_from(vec![
        assign!(TextRepresentation::plain("Bonjour le monde !"), { lang: "fr".into() }),
        assign!(TextRepresentation::plain("Hallo Welt!"), { lang: "de".into() }),
        assign!(TextRepresentation::plain("Hello World!"), { lang: "en".into() }),
    ])
    .unwrap();

    assert_eq!(
        to_json_value(content).unwrap(),
        json!([
            { "body": "Bonjour le monde !", "org.matrix.msc3554.lang": "fr"},
            { "body": "Hallo Welt!", "org.matrix.msc3554.lang": "de"},
            { "body": "Hello World!"},
        ])
    );
}

#[test]
#[cfg(feature = "unstable-msc3554")]
fn lang_deserialization() {
    let json_data = json!([
        { "body": "Bonjour le monde !", "org.matrix.msc3554.lang": "fr"},
        { "body": "Hallo Welt!", "org.matrix.msc3554.lang": "de"},
        { "body": "Hello World!"},
    ]);

    let content = from_json_value::<TextContentBlock>(json_data).unwrap();
    assert_eq!(content[0].lang, "fr");
    assert_eq!(content[1].lang, "de");
    assert_eq!(content[2].lang, "en");
}

#[test]
#[cfg(feature = "unstable-msc3955")]
fn automated_content_serialization() {
    let mut message_event_content =
        MessageEventContent::plain("> <@test:example.com> test\n\ntest reply");
    message_event_content.automated = true;

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                { "body": "> <@test:example.com> test\n\ntest reply" },
            ],
            "org.matrix.msc1767.automated": true,
        })
    );
}

#[test]
#[cfg(feature = "unstable-msc3955")]
fn automated_content_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.text": [
            { "mimetype": "text/html", "body": "Hello, <em>New World</em>!" },
        ],
        "org.matrix.msc1767.automated": true,
    });

    let content = from_json_value::<MessageEventContent>(json_data).unwrap();
    assert_eq!(content.text.find_plain(), None);
    assert_eq!(content.text.find_html(), Some("Hello, <em>New World</em>!"));
    assert!(content.automated);
}
