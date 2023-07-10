use std::borrow::Cow;

use assert_matches2::assert_matches;
use js_int::uint;
use ruma_common::{
    events::{
        key::verification::VerificationMethod,
        room::{
            message::{
                AudioMessageEventContent, EmoteMessageEventContent, FileMessageEventContent,
                ForwardThread, ImageMessageEventContent, KeyVerificationRequestEventContent,
                MessageType, OriginalRoomMessageEvent, OriginalSyncRoomMessageEvent, Relation,
                RoomMessageEventContent, TextMessageEventContent, VideoMessageEventContent,
            },
            EncryptedFileInit, JsonWebKeyInit, MediaSource,
        },
        Mentions, MessageLikeUnsigned,
    },
    mxc_uri, owned_event_id, owned_room_id, owned_user_id,
    serde::Base64,
    user_id, MilliSecondsSinceUnixEpoch, OwnedDeviceId,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

macro_rules! json_object {
    ( $($tt:tt)+ ) => {
        match serde_json::json!({ $($tt)+ }) {
            serde_json::value::Value::Object(map) => map,
            _ => panic!("Not a JSON object"),
        }
    }
}

#[test]
fn custom_msgtype_serialization() {
    let json_data = json_object! {
        "custom_field": "baba",
        "another_one": "abab",
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
fn custom_msgtype_deserialization() {
    let json_data = json!({
        "msgtype": "my_custom_msgtype",
        "body": "my custom message",
        "custom_field": "baba",
        "another_one": "abab",
    });

    let expected_json_data = json_object! {
        "custom_field": "baba",
        "another_one": "abab",
    };

    let custom_event: MessageType = from_json_value(json_data).unwrap();

    assert_eq!(custom_event.msgtype(), "my_custom_msgtype");
    assert_eq!(custom_event.body(), "my custom message");
    assert_eq!(custom_event.data(), Cow::Owned(expected_json_data));
}

#[test]
fn text_msgtype_formatted_body_serialization() {
    let message_event_content =
        RoomMessageEventContent::text_html("Hello, World!", "Hello, <em>World</em>!");

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
fn text_msgtype_plain_text_serialization() {
    let message_event_content =
        RoomMessageEventContent::text_plain("> <@test:example.com> test\n\ntest reply");

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "> <@test:example.com> test\n\ntest reply",
            "msgtype": "m.text"
        })
    );
}

#[test]
#[cfg(feature = "markdown")]
fn text_msgtype_markdown_serialization() {
    use ruma_common::events::room::message::TextMessageEventContent;

    let formatted_message = RoomMessageEventContent::new(MessageType::Text(
        TextMessageEventContent::markdown("Testing **bold** and _italic_!"),
    ));
    assert_eq!(
        to_json_value(&formatted_message).unwrap(),
        json!({
            "body": "Testing **bold** and _italic_!",
            "formatted_body": "<p>Testing <strong>bold</strong> and <em>italic</em>!</p>\n",
            "format": "org.matrix.custom.html",
            "msgtype": "m.text"
        })
    );

    let plain_message_simple = RoomMessageEventContent::new(MessageType::Text(
        TextMessageEventContent::markdown("Testing a simple phrase…"),
    ));
    assert_eq!(
        to_json_value(&plain_message_simple).unwrap(),
        json!({
            "body": "Testing a simple phrase…",
            "msgtype": "m.text"
        })
    );

    let plain_message_paragraphs = RoomMessageEventContent::new(MessageType::Text(
        TextMessageEventContent::markdown("Testing\n\nSeveral\n\nParagraphs."),
    ));
    assert_eq!(
        to_json_value(&plain_message_paragraphs).unwrap(),
        json!({
            "body": "Testing\n\nSeveral\n\nParagraphs.",
            "formatted_body": "<p>Testing</p>\n<p>Several</p>\n<p>Paragraphs.</p>\n",
            "format": "org.matrix.custom.html",
            "msgtype": "m.text"
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
fn verification_request_msgtype_deserialization() {
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

    let content = from_json_value::<RoomMessageEventContent>(json_data).unwrap();

    assert_matches!(content.msgtype, MessageType::VerificationRequest(verification));
    assert_eq!(verification.body, "@example:localhost is requesting to verify your key, ...");
    assert_eq!(verification.to, user_id);
    assert_eq!(verification.from_device, device_id);
    assert_eq!(verification.methods.len(), 3);
    assert!(verification.methods.contains(&VerificationMethod::SasV1));

    assert_matches!(content.relates_to, None);
}

#[test]
fn verification_request_msgtype_serialization() {
    let user_id = owned_user_id!("@example2:localhost");
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
fn content_deserialization_failure() {
    let json_data = json!({
        "body": "test",
        "msgtype": "m.location",
        "url": "http://example.com/audio.mp3"
    });
    assert_matches!(from_json_value::<RoomMessageEventContent>(json_data), Err(_));
}

#[test]
fn escape_tags_in_plain_reply_body() {
    let first_message = OriginalRoomMessageEvent {
        content: RoomMessageEventContent::text_plain("Usage: cp <source> <destination>"),
        event_id: owned_event_id!("$143273582443PhrSn:example.org"),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(10_000)),
        room_id: owned_room_id!("!testroomid:example.org"),
        sender: owned_user_id!("@user:example.org"),
        unsigned: MessageLikeUnsigned::default(),
    };
    let second_message = RoomMessageEventContent::text_plain("Usage: rm <path>")
        .make_reply_to(&first_message, ForwardThread::Yes);

    assert_matches!(
        first_message.content.msgtype,
        MessageType::Text(TextMessageEventContent { body, formatted: None, .. })
    );
    assert_eq!(body, "Usage: cp <source> <destination>");

    assert_matches!(
        second_message.msgtype,
        MessageType::Text(TextMessageEventContent { body, formatted, .. })
    );
    assert_eq!(
        body,
        "\
        > <@user:example.org> Usage: cp <source> <destination>\n\
        \n\
        Usage: rm <path>\
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
                Usage: cp &lt;source&gt; &lt;destination&gt;\
            </blockquote>\
        </mx-reply>\
        Usage: rm &lt;path&gt;\
        "
    );
}

#[test]
#[cfg(feature = "unstable-sanitize")]
fn reply_sanitize() {
    use ruma_common::events::room::message::ForwardThread;

    let first_message = OriginalRoomMessageEvent {
        content: RoomMessageEventContent::text_html(
            "# This is the first message",
            "<h1>This is the first message</h1>",
        ),
        event_id: owned_event_id!("$143273582443PhrSn:example.org"),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(10_000)),
        room_id: owned_room_id!("!testroomid:example.org"),
        sender: owned_user_id!("@user:example.org"),
        unsigned: MessageLikeUnsigned::default(),
    };
    let second_message = OriginalRoomMessageEvent {
        content: RoomMessageEventContent::text_html(
            "This is the _second_ message",
            "This is the <em>second</em> message",
        )
        .make_reply_to(&first_message, ForwardThread::Yes),
        event_id: owned_event_id!("$143273582443PhrSn:example.org"),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(10_000)),
        room_id: owned_room_id!("!testroomid:example.org"),
        sender: owned_user_id!("@user:example.org"),
        unsigned: MessageLikeUnsigned::default(),
    };
    let final_reply = RoomMessageEventContent::text_html(
        "This is **my** reply",
        "This is <strong>my</strong> reply",
    )
    .make_reply_to(&second_message, ForwardThread::Yes);

    assert_matches!(
        first_message.content.msgtype,
        MessageType::Text(TextMessageEventContent { body, formatted, .. })
    );
    assert_eq!(body, "# This is the first message");
    let formatted = formatted.unwrap();
    assert_eq!(formatted.body, "<h1>This is the first message</h1>");

    assert_matches!(
        second_message.content.msgtype,
        MessageType::Text(TextMessageEventContent { body, formatted, .. })
    );
    assert_eq!(
        body,
        "\
        > <@user:example.org> # This is the first message\n\
        \n\
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

    assert_matches!(
        final_reply.msgtype,
        MessageType::Text(TextMessageEventContent { body, formatted, .. })
    );
    assert_eq!(
        body,
        "\
        > <@user:example.org> This is the _second_ message\n\
        \n\
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

#[test]
fn make_replacement_no_reply() {
    let content = RoomMessageEventContent::text_html(
        "This is _an edited_ message.",
        "This is <em>an edited</em> message.",
    );

    let original_message_json = json!({
        "content": {
            "body": "Hello, World!",
            "msgtype": "m.text",
        },
        "event_id": "$143273582443PhrSn",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.room.message",
    });
    let original_message: OriginalSyncRoomMessageEvent =
        from_json_value(original_message_json).unwrap();

    let content = content.make_replacement(&original_message, None);

    assert_matches!(
        content.msgtype,
        MessageType::Text(TextMessageEventContent { body, formatted, .. })
    );
    assert_eq!(body, "* This is _an edited_ message.");
    let formatted = formatted.unwrap();
    assert_eq!(formatted.body, "* This is <em>an edited</em> message.");
    assert_matches!(content.mentions, None);
}

#[test]
fn make_replacement_with_reply() {
    let replied_to_message = OriginalRoomMessageEvent {
        content: RoomMessageEventContent::text_html(
            "# This is the first message",
            "<h1>This is the first message</h1>",
        ),
        event_id: owned_event_id!("$143273582443PhrSn:example.org"),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(10_000)),
        room_id: owned_room_id!("!testroomid:example.org"),
        sender: owned_user_id!("@user:example.org"),
        unsigned: MessageLikeUnsigned::default(),
    };

    let content = RoomMessageEventContent::text_html(
        "This is _an edited_ reply.",
        "This is <em>an edited</em> reply.",
    );

    let original_message_json = json!({
        "content": {
            "body": "Hello, World!",
            "msgtype": "m.text",
        },
        "event_id": "$143273582443PhrSn",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.room.message",
    });
    let original_message: OriginalSyncRoomMessageEvent =
        from_json_value(original_message_json).unwrap();

    let content = content.make_replacement(&original_message, Some(&replied_to_message));

    assert_matches!(
        content.msgtype,
        MessageType::Text(TextMessageEventContent { body, formatted, .. })
    );
    assert_eq!(
        body,
        "\
        > <@user:example.org> # This is the first message\n\
        \n\
        * This is _an edited_ reply.\
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
        * This is <em>an edited</em> reply.\
        "
    );
    assert_matches!(content.mentions, None);
}

#[test]
fn audio_msgtype_serialization() {
    let message_event_content =
        RoomMessageEventContent::new(MessageType::Audio(AudioMessageEventContent::plain(
            "Upload: my_song.mp3".to_owned(),
            mxc_uri!("mxc://notareal.hs/file").to_owned(),
        )));

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "Upload: my_song.mp3",
            "url": "mxc://notareal.hs/file",
            "msgtype": "m.audio",
        })
    );
}

#[test]
fn audio_msgtype_deserialization() {
    let json_data = json!({
        "body": "Upload: my_song.mp3",
        "url": "mxc://notareal.hs/file",
        "msgtype": "m.audio",
    });

    let event_content = from_json_value::<RoomMessageEventContent>(json_data).unwrap();
    assert_matches!(event_content.msgtype, MessageType::Audio(content));
    assert_eq!(content.body, "Upload: my_song.mp3");
    assert_matches!(content.source, MediaSource::Plain(url));
    assert_eq!(url, "mxc://notareal.hs/file");
}

#[test]
fn file_msgtype_plain_content_serialization() {
    let message_event_content =
        RoomMessageEventContent::new(MessageType::File(FileMessageEventContent::plain(
            "Upload: my_file.txt".to_owned(),
            mxc_uri!("mxc://notareal.hs/file").to_owned(),
        )));

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "Upload: my_file.txt",
            "url": "mxc://notareal.hs/file",
            "msgtype": "m.file",
        })
    );
}

#[test]
fn file_msgtype_encrypted_content_serialization() {
    let message_event_content =
        RoomMessageEventContent::new(MessageType::File(FileMessageEventContent::encrypted(
            "Upload: my_file.txt".to_owned(),
            EncryptedFileInit {
                url: mxc_uri!("mxc://notareal.hs/file").to_owned(),
                key: JsonWebKeyInit {
                    kty: "oct".to_owned(),
                    key_ops: vec!["encrypt".to_owned(), "decrypt".to_owned()],
                    alg: "A256CTR".to_owned(),
                    k: Base64::parse("TLlG_OpX807zzQuuwv4QZGJ21_u7weemFGYJFszMn9A").unwrap(),
                    ext: true,
                }
                .into(),
                iv: Base64::parse("S22dq3NAX8wAAAAAAAAAAA").unwrap(),
                hashes: [(
                    "sha256".to_owned(),
                    Base64::parse("aWOHudBnDkJ9IwaR1Nd8XKoI7DOrqDTwt6xDPfVGN6Q").unwrap(),
                )]
                .into(),
                v: "v2".to_owned(),
            }
            .into(),
        )));

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "Upload: my_file.txt",
            "file": {
                "url": "mxc://notareal.hs/file",
                "key": {
                    "kty": "oct",
                    "key_ops": ["encrypt", "decrypt"],
                    "alg": "A256CTR",
                    "k": "TLlG_OpX807zzQuuwv4QZGJ21_u7weemFGYJFszMn9A",
                    "ext": true
                },
                "iv": "S22dq3NAX8wAAAAAAAAAAA",
                "hashes": {
                    "sha256": "aWOHudBnDkJ9IwaR1Nd8XKoI7DOrqDTwt6xDPfVGN6Q"
                },
                "v": "v2",
            },
            "msgtype": "m.file",
        })
    );
}

#[test]
fn file_msgtype_plain_content_deserialization() {
    let json_data = json!({
        "body": "Upload: my_file.txt",
        "url": "mxc://notareal.hs/file",
        "msgtype": "m.file",
    });

    let event_content = from_json_value::<RoomMessageEventContent>(json_data).unwrap();
    assert_matches!(event_content.msgtype, MessageType::File(content));
    assert_eq!(content.body, "Upload: my_file.txt");
    assert_matches!(content.source, MediaSource::Plain(url));
    assert_eq!(url, "mxc://notareal.hs/file");
}

#[test]
fn file_msgtype_encrypted_content_deserialization() {
    let json_data = json!({
        "body": "Upload: my_file.txt",
        "file": {
            "url": "mxc://notareal.hs/file",
            "key": {
                "kty": "oct",
                "key_ops": ["encrypt", "decrypt"],
                "alg": "A256CTR",
                "k": "TLlG_OpX807zzQuuwv4QZGJ21_u7weemFGYJFszMn9A",
                "ext": true
            },
            "iv": "S22dq3NAX8wAAAAAAAAAAA",
            "hashes": {
                "sha256": "aWOHudBnDkJ9IwaR1Nd8XKoI7DOrqDTwt6xDPfVGN6Q"
            },
            "v": "v2",
        },
        "msgtype": "m.file",
    });

    let event_content = from_json_value::<RoomMessageEventContent>(json_data).unwrap();
    assert_matches!(event_content.msgtype, MessageType::File(content));
    assert_eq!(content.body, "Upload: my_file.txt");
    assert_matches!(content.source, MediaSource::Encrypted(encrypted_file));
    assert_eq!(encrypted_file.url, "mxc://notareal.hs/file");
}

#[test]
fn image_msgtype_serialization() {
    let message_event_content =
        RoomMessageEventContent::new(MessageType::Image(ImageMessageEventContent::plain(
            "Upload: my_image.jpg".to_owned(),
            mxc_uri!("mxc://notareal.hs/file").to_owned(),
        )));

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "Upload: my_image.jpg",
            "url": "mxc://notareal.hs/file",
            "msgtype": "m.image",
        })
    );
}

#[test]
fn image_msgtype_deserialization() {
    let json_data = json!({
        "body": "Upload: my_image.jpg",
        "url": "mxc://notareal.hs/file",
        "msgtype": "m.image",
    });

    let event_content = from_json_value::<RoomMessageEventContent>(json_data).unwrap();
    assert_matches!(event_content.msgtype, MessageType::Image(content));
    assert_eq!(content.body, "Upload: my_image.jpg");
    assert_matches!(content.source, MediaSource::Plain(url));
    assert_eq!(url, "mxc://notareal.hs/file");
}

#[cfg(not(feature = "unstable-msc3488"))]
#[test]
fn location_msgtype_serialization() {
    use ruma_common::events::room::message::LocationMessageEventContent;

    let message_event_content =
        RoomMessageEventContent::new(MessageType::Location(LocationMessageEventContent::new(
            "Alice was at geo:51.5008,0.1247;u=35".to_owned(),
            "geo:51.5008,0.1247;u=35".to_owned(),
        )));

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "Alice was at geo:51.5008,0.1247;u=35",
            "geo_uri": "geo:51.5008,0.1247;u=35",
            "msgtype": "m.location",
        })
    );
}

#[test]
fn location_msgtype_deserialization() {
    let json_data = json!({
        "body": "Alice was at geo:51.5008,0.1247;u=35",
        "geo_uri": "geo:51.5008,0.1247;u=35",
        "msgtype": "m.location",
    });

    let event_content = from_json_value::<RoomMessageEventContent>(json_data).unwrap();
    assert_matches!(event_content.msgtype, MessageType::Location(content));
    assert_eq!(content.body, "Alice was at geo:51.5008,0.1247;u=35");
    assert_eq!(content.geo_uri, "geo:51.5008,0.1247;u=35");
}

#[test]
fn text_msgtype_body_deserialization() {
    let json_data = json!({
        "body": "test",
        "msgtype": "m.text",
    });

    assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data),
        Ok(RoomMessageEventContent { msgtype: MessageType::Text(content), .. })
    );
    assert_eq!(content.body, "test");
}

#[test]
fn text_msgtype_formatted_body_and_body_deserialization() {
    let json_data = json!({
        "body": "test",
        "formatted_body": "<h1>test</h1>",
        "format": "org.matrix.custom.html",
        "msgtype": "m.text",
    });

    assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data),
        Ok(RoomMessageEventContent { msgtype: MessageType::Text(content), .. })
    );
    assert_eq!(content.body, "test");
    let formatted = content.formatted.unwrap();
    assert_eq!(formatted.body, "<h1>test</h1>");
}

#[test]
fn notice_msgtype_serialization() {
    let message_event_content =
        RoomMessageEventContent::notice_plain("> <@test:example.com> test\n\ntest reply");

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "> <@test:example.com> test\n\ntest reply",
            "msgtype": "m.notice",
        })
    );
}

#[test]
fn notice_msgtype_deserialization() {
    let json_data = json!({
        "body": "test",
        "msgtype": "m.notice",
    });

    assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data),
        Ok(RoomMessageEventContent { msgtype: MessageType::Notice(content), .. })
    );
    assert_eq!(content.body, "test");
}

#[test]
fn emote_msgtype_serialization() {
    let message_event_content = RoomMessageEventContent::new(MessageType::Emote(
        EmoteMessageEventContent::plain("> <@test:example.com> test\n\ntest reply"),
    ));

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "> <@test:example.com> test\n\ntest reply",
            "msgtype": "m.emote",
        })
    );
}

#[test]
fn emote_msgtype_deserialization() {
    let json_data = json!({
        "body": "test",
        "msgtype": "m.emote",
    });

    assert_matches!(
        from_json_value::<RoomMessageEventContent>(json_data),
        Ok(RoomMessageEventContent { msgtype: MessageType::Emote(content), .. })
    );
    assert_eq!(content.body, "test");
}

#[test]
fn video_msgtype_serialization() {
    let message_event_content =
        RoomMessageEventContent::new(MessageType::Video(VideoMessageEventContent::plain(
            "Upload: my_video.mp4".to_owned(),
            mxc_uri!("mxc://notareal.hs/file").to_owned(),
        )));

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "Upload: my_video.mp4",
            "url": "mxc://notareal.hs/file",
            "msgtype": "m.video",
        })
    );
}

#[test]
fn video_msgtype_deserialization() {
    let json_data = json!({
        "body": "Upload: my_video.mp4",
        "url": "mxc://notareal.hs/file",
        "msgtype": "m.video",
    });

    let event_content = from_json_value::<RoomMessageEventContent>(json_data).unwrap();
    assert_matches!(event_content.msgtype, MessageType::Video(content));
    assert_eq!(content.body, "Upload: my_video.mp4");
    assert_matches!(content.source, MediaSource::Plain(url));
    assert_eq!(url, "mxc://notareal.hs/file");
}

#[test]
fn set_mentions() {
    let mut content = RoomMessageEventContent::text_plain("you!");
    let mentions = content.mentions.take();
    assert_matches!(mentions, None);

    let user_id = owned_user_id!("@you:localhost");
    content = content.set_mentions(Mentions::with_user_ids(vec![user_id.clone()]));
    let mentions = content.mentions.unwrap();
    assert_eq!(mentions.user_ids.as_slice(), &[user_id]);
}

#[test]
fn make_replacement_set_mentions() {
    let alice = owned_user_id!("@alice:localhost");
    let bob = owned_user_id!("@bob:localhost");
    let original_message_json = json!({
        "content": {
            "body": "Hello, World!",
            "msgtype": "m.text",
            "m.mentions": {
                "user_ids": [alice],
            }
        },
        "event_id": "$143273582443PhrSn",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.room.message",
    });
    let original_message: OriginalSyncRoomMessageEvent =
        from_json_value(original_message_json).unwrap();

    let mut content = RoomMessageEventContent::text_html(
        "This is _an edited_ message.",
        "This is <em>an edited</em> message.",
    );
    content = content.make_replacement(&original_message, None);
    let content_clone = content.clone();

    assert_matches!(content.mentions, None);
    assert_matches!(content.relates_to, Some(Relation::Replacement(replacement)));
    let mentions = replacement.new_content.mentions.unwrap();
    assert_eq!(mentions.user_ids.as_slice(), &[alice.clone()]);

    content = content_clone.set_mentions(Mentions::with_user_ids(vec![alice.clone(), bob.clone()]));
    let mentions = content.mentions.unwrap();
    assert_eq!(mentions.user_ids.as_slice(), &[bob.clone()]);
    assert_matches!(content.relates_to, Some(Relation::Replacement(replacement)));
    let mentions = replacement.new_content.mentions.unwrap();
    assert_eq!(mentions.user_ids.as_slice(), &[alice, bob]);
}

#[test]
fn mentions_room_deserialization() {
    let json_data = json!({
        "body": "room!",
        "msgtype": "m.text",
        "m.mentions": {
            "room": true,
        },
    });

    let content = from_json_value::<RoomMessageEventContent>(json_data).unwrap();
    assert_matches!(content.msgtype, MessageType::Text(text));
    assert_eq!(text.body, "room!");
    let mentions = content.mentions.unwrap();
    assert!(mentions.room);
}
