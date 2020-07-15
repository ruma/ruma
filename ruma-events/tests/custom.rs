use std::{
    convert::TryFrom,
    time::{Duration, UNIX_EPOCH},
};

use matches::assert_matches;
use ruma_events::{
    custom::CustomEventContent, AnyMessageEvent, AnyStateEvent, AnyStateEventContent,
    AnySyncMessageEvent, AnySyncRoomEvent, EventJson, MessageEvent, StateEvent, SyncMessageEvent,
    SyncStateEvent, Unsigned,
};
use ruma_identifiers::{EventId, RoomId, UserId};
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
    let aliases_event = AnyMessageEvent::Custom(MessageEvent {
        content: CustomEventContent {
            json: json!({
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
            }),
            event_type: "m.room.message".into(),
        },
        event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(10),
        room_id: RoomId::try_from("!room:room.com").unwrap(),
        sender: UserId::try_from("@carl:example.com").unwrap(),
        unsigned: Unsigned::default(),
    });

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
    let aliases_event = AnyStateEvent::Custom(StateEvent {
        content: CustomEventContent {
            json: json!({
                "custom": 10
            }),
            event_type: "m.made.up".into(),
        },
        event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(10),
        prev_content: None,
        room_id: RoomId::try_from("!roomid:room.com").unwrap(),
        sender: UserId::try_from("@carl:example.com").unwrap(),
        state_key: "".into(),
        unsigned: Unsigned::default(),
    });

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

    let expected_content = json!({
        "m.relates_to": {
            "event_id": "$MDitXXXXXX",
            "key": "👍",
            "rel_type": "m.annotation"
        }
    });

    assert_matches!(
        from_json_value::<EventJson<AnyStateEvent>>(json_data)
            .unwrap()
            .deserialize()
            .unwrap(),
        AnyStateEvent::Custom(StateEvent {
            content: CustomEventContent {
                json, event_type,
            },
            event_id,
            origin_server_ts,
            sender,
            room_id,
            prev_content: None,
            state_key,
            unsigned,
        }) if json == expected_content && event_type == "m.reaction"
            && event_id == EventId::try_from("$h29iv0s8:example.com").unwrap()
            && origin_server_ts == UNIX_EPOCH + Duration::from_millis(10)
            && sender == UserId::try_from("@carl:example.com").unwrap()
            && room_id == RoomId::try_from("!room:room.com").unwrap()
            && state_key == ""
            && !unsigned.is_empty()
    );
}

#[test]
fn deserialize_custom_state_sync_event() {
    let json_data = custom_state_event();

    let expected_content = json!({
        "m.relates_to": {
            "event_id": "$MDitXXXXXX",
            "key": "👍",
            "rel_type": "m.annotation"
        }
    });

    assert_matches!(
        from_json_value::<SyncStateEvent<AnyStateEventContent>>(json_data)
            .unwrap(),
        SyncStateEvent {
            content: AnyStateEventContent::Custom(CustomEventContent {
                json, event_type,
            }),
            event_id,
            origin_server_ts,
            sender,
            prev_content: None,
            state_key,
            unsigned,
        } if json == expected_content && event_type == "m.reaction"
            && event_id == EventId::try_from("$h29iv0s8:example.com").unwrap()
            && origin_server_ts == UNIX_EPOCH + Duration::from_millis(10)
            && sender == UserId::try_from("@carl:example.com").unwrap()
            && state_key == ""
            && !unsigned.is_empty()
    );
}

#[test]
fn deserialize_custom_message_sync_event() {
    let json_data = json!({
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
        "type": "m.reaction",
        "unsigned": {
            "age": 85
        }
    });

    let expected_content = json!({
        "m.relates_to": {
            "event_id": "$MDitXXXXXX",
            "key": "👍",
            "rel_type": "m.annotation"
        }
    });

    assert_matches!(
        from_json_value::<AnySyncRoomEvent>(json_data)
            .unwrap(),
        AnySyncRoomEvent::Message(AnySyncMessageEvent::Custom(SyncMessageEvent {
            content: CustomEventContent {
                json, event_type,
            },
            event_id,
            origin_server_ts,
            sender,
            unsigned,
        })) if json == expected_content && event_type == "m.reaction"
            && event_id == EventId::try_from("$h29iv0s8:example.com").unwrap()
            && origin_server_ts == UNIX_EPOCH + Duration::from_millis(10)
            && sender == UserId::try_from("@carl:example.com").unwrap()
            && !unsigned.is_empty()
    );
}
