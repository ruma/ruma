use std::{collections::BTreeMap, convert::TryFrom};

use maplit::btreemap;
use ruma::{
    events::{
        room::{self},
        AnyStateEvent, AnyStrippedStateEvent, AnySyncStateEvent, EventType,
    },
    identifiers::{EventId, RoomId, RoomVersionId},
};
use serde_json::{from_value as from_json_value, json, Value as JsonValue};
use state_res::{ResolutionResult, StateEvent, StateResolution, StateStore};

// TODO make this an array of events
fn federated_json() -> JsonValue {
    json!({
        "content": {
            "creator": "@example:example.org",
            "m.federate": true,
            "predecessor": {
                "event_id": "$something:example.org",
                "room_id": "!oldroom:example.org"
            },
            "room_version": "6"
        },
        "event_id": "$aaa:example.org",
        "origin_server_ts": 1,
        "room_id": "!room_id:example.org",
        "sender": "@alice:example.org",
        "state_key": "",
        "type": "m.room.create",
        "unsigned": {
            "age": 1234
        }
    })
}

fn room_create() -> JsonValue {
    json!({
        "content": {
            "creator": "@example:example.org",
            "m.federate": true,
            "predecessor": {
                "event_id": "$something:example.org",
                "room_id": "!oldroom:example.org"
            },
            "room_version": "6"
        },
        "event_id": "$aaa:example.org",
        "origin_server_ts": 1,
        "room_id": "!room_id:example.org",
        "sender": "@alice:example.org",
        "state_key": "",
        "type": "m.room.create",
        "unsigned": {
            "age": 1234
        }
    })
}

fn join_rules() -> JsonValue {
    json!({
        "content": {
            "join_rule": "public"
        },
        "event_id": "$bbb:example.org",
        "origin_server_ts": 2,
        "room_id": "!room_id:example.org",
        "sender": "@alice:example.org",
        "state_key": "",
        "type": "m.room.join_rules",
        "unsigned": {
            "age": 1234
        }
    })
}

fn join_event() -> JsonValue {
    json!({
        "content": {
            "avatar_url": null,
            "displayname": "example",
            "membership": "join"
        },
        "event_id": "$ccc:example.org",
        "membership": "join",
        "room_id": "!room_id:example.org",
        "origin_server_ts": 3,
        "sender": "@alice:example.org",
        "state_key": "@alice:example.org",
        "type": "m.room.member",
        "unsigned": {
            "age": 1,
            "replaces_state": "$151800111315tsynI:example.org",
            "prev_content": {
                "avatar_url": null,
                "displayname": "example",
                "membership": "invite"
            }
        }
    })
}

fn power_levels() -> JsonValue {
    json!({
        "content": {
            "ban": 50,
            "events": {
                "m.room.name": 100,
                "m.room.power_levels": 100
            },
            "events_default": 0,
            "invite": 50,
            "kick": 50,
            "notifications": {
                "room": 20
            },
            "redact": 50,
            "state_default": 50,
            "users": {
                "@example:example.org": 100
            },
            "users_default": 0
        },
        "event_id": "$ddd:example.org",
        "origin_server_ts": 4,
        "room_id": "!room_id:example.org",
        "sender": "@example:example.org",
        "state_key": "",
        "type": "m.room.power_levels",
        "unsigned": {
            "age": 1234
        }
    })
}

pub struct TestStore;

impl StateStore for TestStore {
    fn get_events(&self, events: &[EventId]) -> Result<Vec<StateEvent>, String> {
        vec![room_create(), join_rules(), join_event(), power_levels()]
            .into_iter()
            .map(from_json_value)
            .collect::<serde_json::Result<Vec<_>>>()
            .map_err(|e| e.to_string())
    }

    fn get_event(&self, event_id: &EventId) -> Result<StateEvent, String> {
        from_json_value(power_levels()).map_err(|e| e.to_string())
    }

    fn auth_event_ids(&self, room_id: &RoomId, event_id: &EventId) -> Result<Vec<EventId>, String> {
        Ok(vec![
            EventId::try_from("$aaa:example.org").map_err(|e| e.to_string())?
        ])
    }

    fn get_remote_state_for_room(
        &self,
        room_id: &RoomId,
        version: &RoomVersionId,
        event_id: &EventId,
    ) -> Result<(Vec<StateEvent>, Vec<StateEvent>), String> {
        Ok((
            vec![from_json_value(federated_json()).map_err(|e| e.to_string())?],
            vec![from_json_value(power_levels()).map_err(|e| e.to_string())?],
        ))
    }
}

#[test]
fn it_works() {
    let mut store = TestStore;

    let room_id = RoomId::try_from("!room_id:example.org").unwrap();
    let room_version = RoomVersionId::version_6();

    let initial_state = btreemap! {
        (EventType::RoomCreate, "".into()) => EventId::try_from("").unwrap(),
    };

    let state_to_resolve = btreemap! {
        (EventType::RoomCreate, "".into()) => EventId::try_from("").unwrap(),
    };

    let mut resolver = StateResolution::default();

    let res = resolver
        .resolve(&room_id, &room_version, &[initial_state], &mut store)
        .unwrap();
    assert!(if let ResolutionResult::Resolved(_) = res {
        true
    } else {
        false
    });

    let resolved = resolver
        .resolve(&room_id, &room_version, &[state_to_resolve], &mut store)
        .unwrap();

    assert!(resolver.conflicting_events.is_empty());
    assert_eq!(resolver.resolved_events.len(), 3);
    assert_eq!(resolver.resolved_events.len(), 3);
}
