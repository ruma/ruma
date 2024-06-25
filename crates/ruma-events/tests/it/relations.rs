use assert_matches2::assert_matches;
use assign::assign;
use ruma_common::{owned_event_id, serde::Raw};
use ruma_events::{
    relation::{CustomRelation, InReplyTo, Replacement, Thread},
    room::message::{MessageType, Relation, RoomMessageEventContent},
};
use serde_json::{
    from_value as from_json_value, json, to_value as to_json_value, Value as JsonValue,
};

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
        relates_to: Some(Relation::Reply { in_reply_to: InReplyTo::new(owned_event_id!("$1598361704261elfgc")) }),
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
fn reply_serialization_roundtrip() {
    let body = "This is a reply";
    let mut content = RoomMessageEventContent::text_plain(body);
    let reply = InReplyTo::new(owned_event_id!("$1598361704261elfgc"));
    content.relates_to = Some(Relation::Reply { in_reply_to: reply.clone() });

    let json_content = Raw::new(&content).unwrap();
    let deser_content = json_content.deserialize().unwrap();

    assert_matches!(deser_content.msgtype, MessageType::Text(deser_msg));
    assert_eq!(deser_msg.body, body);
    assert_matches!(content.relates_to.unwrap(), Relation::Reply { in_reply_to: deser_reply });
    assert_eq!(deser_reply.event_id, reply.event_id);
}

#[test]
fn replacement_serialize() {
    let content = assign!(
        RoomMessageEventContent::text_plain("<text msg>"),
        {
            relates_to: Some(Relation::Replacement(
                Replacement::new(
                    owned_event_id!("$1598361704261elfgc"),
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
    assert_matches!(replacement.new_content.msgtype, MessageType::Text(text));
    assert_eq!(text.body, "Hello! My name is bar");
}

#[test]
fn replacement_serialization_roundtrip() {
    let body = "<text msg>";
    let mut content = RoomMessageEventContent::text_plain(body);
    let new_body = "This is the new content.";
    let replacement = Replacement::new(
        owned_event_id!("$1598361704261elfgc"),
        RoomMessageEventContent::text_plain(new_body).into(),
    );
    content.relates_to = Some(Relation::Replacement(replacement.clone()));

    let json_content = Raw::new(&content).unwrap();
    let deser_content = json_content.deserialize().unwrap();

    assert_matches!(deser_content.msgtype, MessageType::Text(deser_msg));
    assert_eq!(deser_msg.body, body);
    assert_matches!(content.relates_to.unwrap(), Relation::Replacement(deser_replacement));
    assert_eq!(deser_replacement.event_id, replacement.event_id);
    assert_matches!(deser_replacement.new_content.msgtype, MessageType::Text(deser_new_msg));
    assert_eq!(deser_new_msg.body, new_body);
}

#[test]
fn thread_plain_serialize() {
    let content = assign!(
        RoomMessageEventContent::text_plain("<text msg>"),
        {
            relates_to: Some(Relation::Thread(
                Thread::plain(
                    owned_event_id!("$1598361704261elfgc"),
                    owned_event_id!("$latesteventid"),
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
                    owned_event_id!("$1598361704261elfgc"),
                    owned_event_id!("$repliedtoeventid"),
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

#[test]
fn thread_serialization_roundtrip() {
    let body = "<text msg>";
    let mut content = RoomMessageEventContent::text_plain(body);
    let thread =
        Thread::plain(owned_event_id!("$1598361704261elfgc"), owned_event_id!("$latesteventid"));
    content.relates_to = Some(Relation::Thread(thread.clone()));

    let json_content = Raw::new(&content).unwrap();
    let deser_content = json_content.deserialize().unwrap();

    assert_matches!(deser_content.msgtype, MessageType::Text(deser_msg));
    assert_eq!(deser_msg.body, body);
    assert_matches!(content.relates_to.unwrap(), Relation::Thread(deser_thread));
    assert_eq!(deser_thread.event_id, thread.event_id);
    assert_eq!(deser_thread.in_reply_to.unwrap().event_id, thread.in_reply_to.unwrap().event_id);
    assert_eq!(deser_thread.is_falling_back, thread.is_falling_back);
}

#[test]
fn custom_deserialize() {
    let relation_json = json!({
        "rel_type": "io.ruma.unknown",
        "event_id": "$related_event",
        "key": "value",
    });
    let content_json = json!({
        "msgtype": "m.text",
        "body": "<text msg>",
        "m.relates_to": relation_json,
    });

    assert_matches!(
        from_json_value::<RoomMessageEventContent>(content_json),
        Ok(RoomMessageEventContent {
            msgtype: MessageType::Text(_),
            relates_to: Some(relation),
            ..
        })
    );
    assert_eq!(relation.rel_type().unwrap().as_str(), "io.ruma.unknown");
    assert_eq!(JsonValue::Object(relation.data().into_owned()), relation_json);
}

#[test]
fn custom_serialize() {
    let json = json!({
        "rel_type": "io.ruma.unknown",
        "event_id": "$related_event",
        "key": "value",
    });
    let relation = from_json_value::<CustomRelation>(json).unwrap();

    let mut content = RoomMessageEventContent::text_plain("<text msg>");
    content.relates_to = Some(Relation::_Custom(relation));

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "msgtype": "m.text",
            "body": "<text msg>",
            "m.relates_to": {
                "rel_type": "io.ruma.unknown",
                "event_id": "$related_event",
                "key": "value",
            },
        })
    );
}

#[test]
fn custom_serialization_roundtrip() {
    let rel_type = "io.ruma.unknown";
    let event_id = "$related_event";
    let key = "value";
    let json_relation = json!({
        "rel_type": rel_type,
        "event_id": event_id,
        "key": key,
    });
    let relation = from_json_value::<CustomRelation>(json_relation).unwrap();

    let body = "<text msg>";
    let mut content = RoomMessageEventContent::text_plain(body);
    content.relates_to = Some(Relation::_Custom(relation));

    let json_content = Raw::new(&content).unwrap();
    let deser_content = json_content.deserialize().unwrap();

    assert_matches!(deser_content.msgtype, MessageType::Text(deser_msg));
    assert_eq!(deser_msg.body, body);
    let deser_relates_to = deser_content.relates_to.unwrap();
    assert_matches!(&deser_relates_to, Relation::_Custom(_));
    assert_eq!(deser_relates_to.rel_type().unwrap().as_str(), rel_type);
    let deser_relation = deser_relates_to.data();
    assert_eq!(deser_relation.get("rel_type").unwrap().as_str().unwrap(), rel_type);
    assert_eq!(deser_relation.get("event_id").unwrap().as_str().unwrap(), event_id);
    assert_eq!(deser_relation.get("key").unwrap().as_str().unwrap(), key);
}
