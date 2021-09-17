use js_int::uint;
use maplit::btreemap;
use matches::assert_matches;
use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_events::{
    custom::CustomEventContent, AnyStateEvent, AnySyncRoomEvent, AnySyncStateEvent, MessageEvent,
    StateEvent, Unsigned,
};
use ruma_identifiers::{event_id, room_id, user_id};
use serde_json::{
    from_value as from_json_value, json, to_value as to_json_value, Value as JsonValue,
};

fn custom_state_event() -> JsonValue {
    json!({
        "content": {
            "m.relates_to": {
                "event_id": "$MDitXXXXXX",
                "key": "👍",
                "rel_type": "m.annotation"
            }
        },
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 10,
        "room_id": "!room:room.com",
        "sender": "@carl:example.com",
        "state_key": "",
        "type": "m.reaction",
        "unsigned": {
            "age": 85
        }
    })
}

#[test]
fn serialize_custom_message_event() {
    let aliases_event = MessageEvent {
        content: CustomEventContent {
            data: btreemap! {
                "body".into() => " * edited message".into(),
                "m.new_content".into() => json!({
                    "body": "edited message",
                    "msgtype": "m.text"
                }),
                "m.relates_to".into() => json!({
                    "event_id": "some event id",
                    "rel_type": "m.replace"
                }),
                "msgtype".into() => "m.text".into()
            },
            event_type: "m.room.message".into(),
        },
        event_id: event_id!("$h29iv0s8:example.com").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(10)),
        room_id: room_id!("!room:room.com"),
        sender: user_id!("@carl:example.com"),
        unsigned: Unsigned::default(),
    };

    let actual = to_json_value(&aliases_event).unwrap();
    let expected = json!({
        "content": {
            "body": " * edited message",
            "m.new_content": {
                "body": "edited message",
                "msgtype": "m.text"
            },
            "m.relates_to": {
                "event_id": "some event id",
                "rel_type": "m.replace"
            },
            "msgtype": "m.text"
        },
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 10,
        "sender": "@carl:example.com",
        "room_id": "!room:room.com",
        "type": "m.room.message",
    });

    assert_eq!(actual, expected);
}

#[test]
fn serialize_custom_state_event() {
    let aliases_event = StateEvent {
        content: CustomEventContent {
            data: btreemap! {
                "custom".into() => 10.into()
            },
            event_type: "m.made.up".into(),
        },
        event_id: event_id!("$h29iv0s8:example.com").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(10)),
        prev_content: None,
        room_id: room_id!("!roomid:room.com"),
        sender: user_id!("@carl:example.com"),
        state_key: "".into(),
        unsigned: Unsigned::default(),
    };

    let actual = to_json_value(&aliases_event).unwrap();
    let expected = json!({
        "content": {
            "custom": 10
        },
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 10,
        "room_id": "!roomid:room.com",
        "sender": "@carl:example.com",
        "state_key": "",
        "type": "m.made.up",
    });

    assert_eq!(actual, expected);
}

#[test]
fn deserialize_custom_state_event() {
    let json_data = custom_state_event();
    assert_matches!(from_json_value::<AnyStateEvent>(json_data), Ok(_));
}

#[test]
fn deserialize_custom_state_sync_event() {
    let json_data = custom_state_event();
    assert_matches!(from_json_value::<AnySyncStateEvent>(json_data), Ok(_));
}

#[test]
fn deserialize_custom_message_sync_event() {
    let json_data = json!({
        "content": {
            "body": "👍"
        },
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 10,
        "room_id": "!room:room.com",
        "sender": "@carl:example.com",
        "type": "m.ruma_custom",
        "unsigned": {
            "age": 85
        }
    });

    assert_matches!(
        from_json_value::<AnySyncRoomEvent>(json_data),
        Ok(AnySyncRoomEvent::Message(_))
    );
}
