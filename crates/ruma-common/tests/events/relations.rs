use assert_matches::assert_matches;
use assign::assign;
use ruma_common::{
    event_id,
    events::room::message::{InReplyTo, MessageType, Relation, RoomMessageEventContent},
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn reply_deserialize() {
    let ev_id = event_id!("$1598361704261elfgc:localhost");

    let json = json!({
        "msgtype": "m.text",
        "body": "<text msg>",
        "m.relates_to": {
            "m.in_reply_to": {
                "event_id": ev_id,
            },
        },
    });

    assert_matches!(
        from_json_value::<RoomMessageEventContent>(json).unwrap(),
        RoomMessageEventContent {
            msgtype: MessageType::Text(_),
            relates_to: Some(Relation::Reply { in_reply_to: InReplyTo { event_id, .. }, .. }),
            ..
        } if event_id == ev_id
    );
}

#[test]
fn reply_serialize() {
    let content = assign!(RoomMessageEventContent::text_plain("This is a reply"), {
        relates_to: Some(Relation::Reply { in_reply_to: InReplyTo::new(event_id!("$1598361704261elfgc").to_owned()) }),
    });

    #[cfg(not(feature = "unstable-msc1767"))]
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

    #[cfg(feature = "unstable-msc1767")]
    assert_eq!(
        to_json_value(content).unwrap(),
        json!({
            "msgtype": "m.text",
            "body": "This is a reply",
            "org.matrix.msc1767.text": "This is a reply",
            "m.relates_to": {
                "m.in_reply_to": {
                    "event_id": "$1598361704261elfgc",
                },
            },
        })
    );
}

#[test]
#[cfg(feature = "unstable-msc2676")]
fn replacement_serialize() {
    use ruma_common::events::room::message::Replacement;

    let content = assign!(
        RoomMessageEventContent::text_plain("<text msg>"),
        {
            relates_to: Some(Relation::Replacement(
                Replacement::new(
                    event_id!("$1598361704261elfgc").to_owned(),
                    Box::new(RoomMessageEventContent::text_plain("This is the new content.")),
                )
            ))
        }
    );

    #[cfg(not(feature = "unstable-msc1767"))]
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

    #[cfg(feature = "unstable-msc1767")]
    assert_eq!(
        to_json_value(content).unwrap(),
        json!({
            "msgtype": "m.text",
            "body": "<text msg>",
            "org.matrix.msc1767.text": "<text msg>",
            "m.new_content": {
                "body": "This is the new content.",
                "msgtype": "m.text",
                "org.matrix.msc1767.text": "This is the new content.",
            },
            "m.relates_to": {
                "rel_type": "m.replace",
                "event_id": "$1598361704261elfgc",
            },
        })
    );
}

#[test]
#[cfg(feature = "unstable-msc2676")]
fn replacement_deserialize() {
    use ruma_common::events::room::message::Replacement;

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
        from_json_value::<RoomMessageEventContent>(json).unwrap(),
        RoomMessageEventContent {
            msgtype: MessageType::Text(_),
            relates_to: Some(Relation::Replacement(Replacement { event_id, new_content, .. })),
            ..
        } if event_id == "$1598361704261elfgc"
          && matches!(&new_content.msgtype, MessageType::Text(text) if text.body == "Hello! My name is bar")
    );
}

#[test]
#[cfg(feature = "unstable-msc3440")]
fn thread_plain_serialize() {
    use ruma_common::events::room::message::Thread;

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

    #[cfg(not(feature = "unstable-msc1767"))]
    assert_eq!(
        to_json_value(content).unwrap(),
        json!({
            "msgtype": "m.text",
            "body": "<text msg>",
            "m.relates_to": {
                "rel_type": "io.element.thread",
                "event_id": "$1598361704261elfgc",
                "m.in_reply_to": {
                    "event_id": "$latesteventid",
                },
                "io.element.show_reply": true,
            },
        })
    );

    #[cfg(feature = "unstable-msc1767")]
    assert_eq!(
        to_json_value(content).unwrap(),
        json!({
            "msgtype": "m.text",
            "body": "<text msg>",
            "org.matrix.msc1767.text": "<text msg>",
            "m.relates_to": {
                "rel_type": "io.element.thread",
                "event_id": "$1598361704261elfgc",
                "m.in_reply_to": {
                    "event_id": "$latesteventid",
                },
                "io.element.show_reply": true,
            },
        })
    );
}

#[test]
#[cfg(feature = "unstable-msc3440")]
fn thread_reply_serialize() {
    use ruma_common::events::room::message::Thread;

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

    #[cfg(not(feature = "unstable-msc1767"))]
    assert_eq!(
        to_json_value(content).unwrap(),
        json!({
            "msgtype": "m.text",
            "body": "<text msg>",
            "m.relates_to": {
                "rel_type": "io.element.thread",
                "event_id": "$1598361704261elfgc",
                "m.in_reply_to": {
                    "event_id": "$repliedtoeventid",
                },
            },
        })
    );

    #[cfg(feature = "unstable-msc1767")]
    assert_eq!(
        to_json_value(content).unwrap(),
        json!({
            "msgtype": "m.text",
            "body": "<text msg>",
            "org.matrix.msc1767.text": "<text msg>",
            "m.relates_to": {
                "rel_type": "io.element.thread",
                "event_id": "$1598361704261elfgc",
                "m.in_reply_to": {
                    "event_id": "$repliedtoeventid",
                },
            },
        })
    );
}

#[test]
#[cfg(feature = "unstable-msc3440")]
fn thread_stable_deserialize() {
    use ruma_common::events::room::message::Thread;

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
        from_json_value::<RoomMessageEventContent>(json).unwrap(),
        RoomMessageEventContent {
            msgtype: MessageType::Text(_),
            relates_to: Some(Relation::Thread(
                Thread {
                    event_id,
                    in_reply_to: InReplyTo { event_id: reply_to_event_id, .. },
                    is_falling_back,
                    ..
                },
            )),
            ..
        } if event_id == "$1598361704261elfgc"
          && reply_to_event_id == "$latesteventid"
          && !is_falling_back
    );
}

#[test]
#[cfg(feature = "unstable-msc3440")]
fn thread_unstable_deserialize() {
    use ruma_common::events::room::message::Thread;

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
        from_json_value::<RoomMessageEventContent>(json).unwrap(),
        RoomMessageEventContent {
            msgtype: MessageType::Text(_),
            relates_to: Some(Relation::Thread(
                Thread {
                    event_id,
                    in_reply_to: InReplyTo { event_id: reply_to_event_id, .. },
                    is_falling_back,
                    ..
                },
            )),
            ..
        } if event_id == "$1598361704261elfgc"
          && reply_to_event_id == "$latesteventid"
          && !is_falling_back
    );
}
