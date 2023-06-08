use assert_matches2::assert_matches;
use assign::assign;
use ruma_common::{
    event_id,
    events::{
        relation::{InReplyTo, Replacement, Thread},
        room::message::{MessageType, Relation, RoomMessageEventContent},
    },
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn reply_deserialize() {
    let json = json!({
        "msgtype": "m.text",
        "body": "<text msg>",
        "m.relates_to": {
            "m.in_reply_to": {
                "event_id": "$1598361704261elfgc:localhost",
            },
        },
    });

    assert_matches!(
        from_json_value::<RoomMessageEventContent>(json),
        Ok(RoomMessageEventContent {
            msgtype: MessageType::Text(_),
            relates_to: Some(Relation::Reply { in_reply_to: InReplyTo { event_id, .. }, .. }),
            ..
        })
    );
    assert_eq!(event_id, "$1598361704261elfgc:localhost");
}

#[test]
fn reply_serialize() {
    let content = assign!(RoomMessageEventContent::text_plain("This is a reply"), {
        relates_to: Some(Relation::Reply { in_reply_to: InReplyTo::new(event_id!("$1598361704261elfgc").to_owned()) }),
    });

    assert_eq!(
        to_json_value(content).unwrap(),
        json!({
            "msgtype": "m.text",
            "body": "This is a reply",
            "m.relates_to": {
                "m.in_reply_to": {
                    "event_id": "$1598361704261elfgc",
                },
            },
        })
    );
}

#[test]
fn replacement_serialize() {
    let content = assign!(
        RoomMessageEventContent::text_plain("<text msg>"),
        {
            relates_to: Some(Relation::Replacement(
                Replacement::new(
                    event_id!("$1598361704261elfgc").to_owned(),
                    RoomMessageEventContent::text_plain("This is the new content.").into(),
                )
            ))
        }
    );

    assert_eq!(
        to_json_value(content).unwrap(),
        json!({
            "msgtype": "m.text",
            "body": "<text msg>",
            "m.new_content": {
                "body": "This is the new content.",
                "msgtype": "m.text",
            },
            "m.relates_to": {
                "rel_type": "m.replace",
                "event_id": "$1598361704261elfgc",
            },
        })
    );
}

#[test]
fn replacement_deserialize() {
    let json = json!({
        "msgtype": "m.text",
        "body": "<text msg>",
        "m.new_content": {
            "body": "Hello! My name is bar",
            "msgtype": "m.text",
        },
        "m.relates_to": {
            "rel_type": "m.replace",
            "event_id": "$1598361704261elfgc",
        },
    });

    assert_matches!(
        from_json_value::<RoomMessageEventContent>(json),
        Ok(RoomMessageEventContent {
            msgtype: MessageType::Text(_),
            relates_to: Some(Relation::Replacement(replacement)),
            ..
        })
    );
    assert_eq!(replacement.event_id, "$1598361704261elfgc");
    assert_matches!(replacement.new_content, MessageType::Text(text));
    assert_eq!(text.body, "Hello! My name is bar");
}

#[test]
fn thread_plain_serialize() {
    let content = assign!(
        RoomMessageEventContent::text_plain("<text msg>"),
        {
            relates_to: Some(Relation::Thread(
                Thread::plain(
                    event_id!("$1598361704261elfgc").to_owned(),
                    event_id!("$latesteventid").to_owned(),
                ),
            )),
        }
    );

    assert_eq!(
        to_json_value(content).unwrap(),
        json!({
            "msgtype": "m.text",
            "body": "<text msg>",
            "m.relates_to": {
                "rel_type": "m.thread",
                "event_id": "$1598361704261elfgc",
                "m.in_reply_to": {
                    "event_id": "$latesteventid",
                },
                "is_falling_back": true,
            },
        })
    );
}

#[test]
fn thread_reply_serialize() {
    let content = assign!(
        RoomMessageEventContent::text_plain("<text msg>"),
        {
            relates_to: Some(Relation::Thread(
                Thread::reply(
                    event_id!("$1598361704261elfgc").to_owned(),
                    event_id!("$repliedtoeventid").to_owned(),
                ),
            )),
        }
    );

    assert_eq!(
        to_json_value(content).unwrap(),
        json!({
            "msgtype": "m.text",
            "body": "<text msg>",
            "m.relates_to": {
                "rel_type": "m.thread",
                "event_id": "$1598361704261elfgc",
                "m.in_reply_to": {
                    "event_id": "$repliedtoeventid",
                },
            },
        })
    );
}

#[test]
fn thread_stable_deserialize() {
    let json = json!({
        "msgtype": "m.text",
        "body": "<text msg>",
        "m.relates_to": {
            "rel_type": "m.thread",
            "event_id": "$1598361704261elfgc",
        },
    });

    assert_matches!(
        from_json_value::<RoomMessageEventContent>(json),
        Ok(RoomMessageEventContent {
            msgtype: MessageType::Text(_),
            relates_to: Some(Relation::Thread(thread)),
            ..
        })
    );
    assert_eq!(thread.event_id, "$1598361704261elfgc");
    assert_matches!(thread.in_reply_to, None);
    assert!(!thread.is_falling_back);
}

#[test]
fn thread_stable_reply_deserialize() {
    let json = json!({
        "msgtype": "m.text",
        "body": "<text msg>",
        "m.relates_to": {
            "rel_type": "m.thread",
            "event_id": "$1598361704261elfgc",
            "m.in_reply_to": {
                "event_id": "$latesteventid",
            },
        },
    });

    assert_matches!(
        from_json_value::<RoomMessageEventContent>(json),
        Ok(RoomMessageEventContent {
            msgtype: MessageType::Text(_),
            relates_to: Some(Relation::Thread(thread)),
            ..
        })
    );
    assert_eq!(thread.event_id, "$1598361704261elfgc");
    assert_eq!(thread.in_reply_to.unwrap().event_id, "$latesteventid");
    assert!(!thread.is_falling_back);
}

#[test]
fn thread_unstable_deserialize() {
    let json = json!({
        "msgtype": "m.text",
        "body": "<text msg>",
        "m.relates_to": {
            "rel_type": "io.element.thread",
            "event_id": "$1598361704261elfgc",
            "m.in_reply_to": {
                "event_id": "$latesteventid",
            },
        },
    });

    assert_matches!(
        from_json_value::<RoomMessageEventContent>(json),
        Ok(RoomMessageEventContent {
            msgtype: MessageType::Text(_),
            relates_to: Some(Relation::Thread(thread)),
            ..
        })
    );
    assert_eq!(thread.event_id, "$1598361704261elfgc");
    assert_eq!(thread.in_reply_to.unwrap().event_id, "$latesteventid");
    assert!(!thread.is_falling_back);
}
