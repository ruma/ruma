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

// See note about wrapping macro expansion in a module from `src/lib.rs`
mod common_case {
    use super::*;

    ruma_event! {
        /// Informs the room about what room aliases it has been given.
        AliasesEvent {
            kind: StateEvent,
            event_type: "m.room.aliases",
            content: {
                /// A list of room aliases.
                pub aliases: Vec<ruma_identifiers::RoomAliasId>,
            }
        }
    }

    #[test]
    fn optional_fields_as_none() {
        let json = json!({
            "content": {
                "aliases": []
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "sender": "@carl:example.com",
            "state_key": "example.com",
            "type": "m.room.aliases"
        });

        assert_matches!(
            from_json_value::<EventJson<AliasesEvent>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            AliasesEvent {
                content: AliasesEventContent { aliases },
                event_id,
                origin_server_ts,
                prev_content: None,
                room_id: None,
                sender,
                state_key,
                unsigned,
            } if aliases.is_empty()
                && event_id == "$h29iv0s8:example.com"
                && origin_server_ts == UNIX_EPOCH + Duration::from_millis(1)
                && sender == "@carl:example.com"
                && state_key == "example.com"
                && unsigned.is_empty()
        )
    }

    #[test]
    fn some_optional_fields_as_some() {
        let json = json!({
            "content": {
                "aliases": ["#room:example.org"]
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "prev_content": {
                "aliases": []
            },
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "example.com",
            "type": "m.room.aliases"
        });

        assert_matches!(
            from_json_value::<EventJson<AliasesEvent>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            AliasesEvent {
                content: AliasesEventContent { aliases, },
                event_id,
                origin_server_ts,
                prev_content: Some(AliasesEventContent { aliases: prev_aliases }),
                room_id: Some(room_id),
                sender,
                state_key,
                unsigned,
            } if aliases == vec![RoomAliasId::try_from("#room:example.org").unwrap()]
                && event_id == "$h29iv0s8:example.com"
                && origin_server_ts == UNIX_EPOCH + Duration::from_millis(1)
                && prev_aliases.is_empty()
                && room_id == "!n8f893n9:example.com"
                && sender == "@carl:example.com"
                && state_key == "example.com"
                && unsigned.is_empty()
        );
    }

    #[test]
    fn all_optional_fields_as_some() {
        let json = json!({
            "content": {
                "aliases": ["#room:example.org"]
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "prev_content": {
                "aliases": []
            },
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "example.com",
            "unsigned": {
                "age": 100
            },
            "type": "m.room.aliases"
        });

        assert_matches!(
            from_json_value::<EventJson<AliasesEvent>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            AliasesEvent {
                content: AliasesEventContent { aliases },
                event_id,
                origin_server_ts,
                prev_content: Some(AliasesEventContent { aliases: prev_aliases }),
                room_id: Some(room_id),
                sender,
                state_key,
                unsigned: UnsignedData {
                    age: Some(age),
                    redacted_because: None,
                    transaction_id: None,
                },
            } if aliases == vec![RoomAliasId::try_from("#room:example.org").unwrap()]
                && event_id == "$h29iv0s8:example.com"
                && origin_server_ts == UNIX_EPOCH + Duration::from_millis(1)
                && prev_aliases.is_empty()
                && room_id == "!n8f893n9:example.com"
                && sender == "@carl:example.com"
                && state_key == "example.com"
                && age == Int::from(100)
        );
    }
}

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
