use std::{
    collections::BTreeMap,
    convert::TryFrom,
    time::{Duration, UNIX_EPOCH},
};

use js_int::Int;
use maplit::btreemap;
use matches::assert_matches;
use ruma_events::{EventJson, UnsignedData};
use ruma_events_macros::ruma_event;
use ruma_identifiers::{RoomAliasId, RoomId, UserId};
use serde_json::{from_value as from_json_value, json};

mod extra_fields {
    use super::*;

    ruma_event! {
        /// A redaction of an event.
        RedactionEvent {
            kind: RoomEvent,
            event_type: "m.room.redaction",
            fields: {
                /// The ID of the event that was redacted.
                pub redacts: ruma_identifiers::EventId
            },
            content: {
                /// The reason for the redaction, if any.
                pub reason: Option<String>,
            },
        }
    }

    #[test]
    fn field_serialization_deserialization() {
        let json = json!({
            "content": {
                "reason": null
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "redacts": "$h29iv0s8:example.com",
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "unsigned": {
                "age": 100
            },
            "type": "m.room.redaction"
        });

        assert_matches!(
            from_json_value::<EventJson<RedactionEvent>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            RedactionEvent {
                content: RedactionEventContent { reason: None },
                redacts,
                event_id,
                origin_server_ts,
                room_id: Some(room_id),
                sender,
                unsigned: UnsignedData {
                    age: Some(age),
                    redacted_because: None,
                    transaction_id: None,
                },
            } if redacts == "$h29iv0s8:example.com"
                && event_id == "$h29iv0s8:example.com"
                && origin_server_ts == UNIX_EPOCH + Duration::from_millis(1)
                && room_id == "!n8f893n9:example.com"
                && sender == "@carl:example.com"
                && age == Int::from(100)
        );
    }
}

mod type_alias {
    use super::*;

    ruma_event! {
        /// Informs the client about the rooms that are considered direct by a user.
        DirectEvent {
            kind: Event,
            event_type: "m.direct",
            content_type_alias: {
                /// The payload of a `DirectEvent`.
                ///
                /// A mapping of `UserId`'s to a collection of `RoomId`'s which are considered
                /// *direct* for that particular user.
                BTreeMap<ruma_identifiers::UserId, Vec<ruma_identifiers::RoomId>>
            }
        }
    }

    #[test]
    fn alias_is_not_empty() {
        let json = json!({
            "content": {
                "@bob:example.com": ["!n8f893n9:example.com"]
            },
            "type": "m.direct"
        });

        let event = from_json_value::<EventJson<DirectEvent>>(json)
            .unwrap()
            .deserialize()
            .unwrap();

        assert_eq!(
            event.content,
            btreemap! {
                UserId::try_from("@bob:example.com").unwrap() => vec![
                    RoomId::try_from("!n8f893n9:example.com").unwrap()
                ]
            }
        );
    }

    #[test]
    fn alias_empty() {
        let json = json!({
            "content": {},
            "type": "m.direct"
        });

        let _ = from_json_value::<EventJson<DirectEvent>>(json)
            .unwrap()
            .deserialize()
            .unwrap();
    }
}
