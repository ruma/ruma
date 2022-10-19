use std::borrow::Cow;

use assert_matches::assert_matches;
use js_int::uint;
use ruma_common::{
    event_id,
    events::{
        key::verification::VerificationMethod,
        room::{
            message::{
                AudioMessageEventContent, KeyVerificationRequestEventContent, MessageType,
                OriginalRoomMessageEvent, RoomMessageEventContent,
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
            "org.matrix.msc1767.html": "Hello, <em>World</em>!",
            "org.matrix.msc1767.text": "Hello, World!",
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
    use ruma_common::events::room::message::TextMessageEventContent;

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
            "org.matrix.msc1767.html": "<p>Testing <strong>bold</strong> and <em>italic</em>!</p>\n",
            "org.matrix.msc1767.text": "Testing **bold** and _italic_!",
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
            "org.matrix.msc1767.html": "<p>Testing</p>\n<p>Several</p>\n<p>Paragraphs.</p>\n",
            "org.matrix.msc1767.text": "Testing\n\nSeveral\n\nParagraphs.",
        })
    );
}

#[test]
#[cfg(feature = "markdown")]
fn markdown_detection() {
    use ruma_common::events::room::message::FormattedBody;

    // No markdown
    let formatted_body = FormattedBody::markdown("A simple message.");
    assert_matches!(formatted_body, None);

    // Multiple paragraphs trigger markdown
    let formatted_body = FormattedBody::markdown("A message\nwith\n\nmultiple\n\nparagraphs");
    formatted_body.unwrap();

    // HTML entities don't trigger markdown.
    let formatted_body = FormattedBody::markdown("A message with & HTML < entities");
    assert_matches!(formatted_body, None);

    // HTML triggers markdown.
    let formatted_body = FormattedBody::markdown("<span>An HTML message</span>");
    formatted_body.unwrap();
}

#[test]
#[cfg(feature = "markdown")]
fn markdown_options() {
    use ruma_common::events::room::message::FormattedBody;

    // Tables
    let formatted_body = FormattedBody::markdown(
        "|head1|head2|\n\
        |---|---|\n\
        |body1|body2|\
        ",
    );
    assert_eq!(
        formatted_body.unwrap().body,
        "<table>\
            <thead><tr><th>head1</th><th>head2</th></tr></thead>\
            <tbody>\n<tr><td>body1</td><td>body2</td></tr>\n</tbody>\
        </table>\n"
    );

    // Strikethrough
    let formatted_body = FormattedBody::markdown("A message with a ~~strike~~");
    assert_eq!(formatted_body.unwrap().body, "<p>A message with a <del>strike</del></p>\n");
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

    let verification = assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data),
        Ok(RoomMessageEventContent {
            msgtype: MessageType::VerificationRequest(verification),
            ..
        }) => verification
    );
    assert_eq!(verification.body, "@example:localhost is requesting to verify your key, ...");
    assert_eq!(verification.to, user_id);
    assert_eq!(verification.from_device, device_id);
    assert_eq!(verification.methods.len(), 3);
    assert!(verification.methods.contains(&VerificationMethod::SasV1));
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

    let audio = assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data),
        Ok(RoomMessageEventContent {
            msgtype: MessageType::Audio(audio),
            ..
        }) => audio
    );
    assert_eq!(audio.body, "test");
    assert_matches!(audio.info, None);
    let url = assert_matches!(audio.source, MediaSource::Plain(url) => url);
    assert_eq!(url, "mxc://example.org/ffed755USFFxlgbQYZGtryd");
}

#[test]
fn content_deserialization_failure() {
    let json_data = json!({
        "body": "test","msgtype": "m.location",
        "url": "http://example.com/audio.mp3"
    });
    assert_matches!(from_json_value::<RoomMessageEventContent>(json_data), Err(_));
}

#[test]
#[cfg(feature = "unstable-sanitize")]
fn reply_sanitize() {
    use ruma_common::events::room::message::{ForwardThread, TextMessageEventContent};

    let first_message = OriginalRoomMessageEvent {
        content: RoomMessageEventContent::text_html(
            "# This is the first message",
            "<h1>This is the first message</h1>",
        ),
        event_id: event_id!("$143273582443PhrSn:example.org").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(10_000)),
        room_id: room_id!("!testroomid:example.org").to_owned(),
        sender: user_id!("@user:example.org").to_owned(),
        unsigned: MessageLikeUnsigned::default(),
    };
    let second_message = OriginalRoomMessageEvent {
        content: RoomMessageEventContent::text_html(
            "This is the _second_ message",
            "This is the <em>second</em> message",
        )
        .make_reply_to(&first_message, ForwardThread::Yes),
        event_id: event_id!("$143273582443PhrSn:example.org").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(10_000)),
        room_id: room_id!("!testroomid:example.org").to_owned(),
        sender: user_id!("@user:example.org").to_owned(),
        unsigned: MessageLikeUnsigned::default(),
    };
    let final_reply = RoomMessageEventContent::text_html(
        "This is **my** reply",
        "This is <strong>my</strong> reply",
    )
    .make_reply_to(&second_message, ForwardThread::Yes);

    let (body, formatted) = assert_matches!(
        first_message.content.msgtype,
        MessageType::Text(TextMessageEventContent { body, formatted, .. }) => (body, formatted)
    );
    assert_eq!(body, "# This is the first message");
    let formatted = formatted.unwrap();
    assert_eq!(formatted.body, "<h1>This is the first message</h1>");

    let (body, formatted) = assert_matches!(
        second_message.content.msgtype,
        MessageType::Text(TextMessageEventContent { body, formatted, .. }) => (body, formatted)
    );
    assert_eq!(
        body,
        "\
        > <@user:example.org> # This is the first message\n\
        This is the _second_ message\
        "
    );
    let formatted = formatted.unwrap();
    assert_eq!(
        formatted.body,
        "\
        <mx-reply>\
            <blockquote>\
                <a href=\"https://matrix.to/#/!testroomid:example.org/$143273582443PhrSn:example.org\">In reply to</a> \
                <a href=\"https://matrix.to/#/@user:example.org\">@user:example.org</a>\
                <br>\
                <h1>This is the first message</h1>\
            </blockquote>\
        </mx-reply>\
        This is the <em>second</em> message\
        "
    );

    let (body, formatted) = assert_matches!(
        final_reply.msgtype,
        MessageType::Text(TextMessageEventContent { body, formatted, .. }) => (body, formatted)
    );
    assert_eq!(
        body,
        "\
        > <@user:example.org> This is the _second_ message\n\
        This is **my** reply\
        "
    );
    let formatted = formatted.unwrap();
    assert_eq!(
        formatted.body,
        "\
        <mx-reply>\
            <blockquote>\
                <a href=\"https://matrix.to/#/!testroomid:example.org/$143273582443PhrSn:example.org\">In reply to</a> \
                <a href=\"https://matrix.to/#/@user:example.org\">@user:example.org</a>\
                <br>\
                This is the <em>second</em> message\
            </blockquote>\
        </mx-reply>\
        This is <strong>my</strong> reply\
        "
    );
}
