use std::{collections::HashMap, convert::TryFrom};

use js_int::UInt;
use ruma_events::util::serde_json_eq_try_from_raw;
use ruma_events_macros::ruma_event;
use ruma_identifiers::{EventId, RoomAliasId, RoomId, UserId};
use serde_json::{json, Value};

// See note about wrapping macro expansion in a module from `src/lib.rs`
mod common_case {
    use super::*;

    ruma_event! {
        /// Informs the room about what room aliases it has been given.
        AliasesEvent {
            kind: StateEvent,
            event_type: RoomAliases,
            content: {
                /// A list of room aliases.
                pub aliases: Vec<ruma_identifiers::RoomAliasId>,
            }
        }
    }

    #[test]
    fn optional_fields_as_none() {
        let event = AliasesEvent {
            content: AliasesEventContent {
                aliases: Vec::with_capacity(0),
            },
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UInt::try_from(1).unwrap(),
            prev_content: None,
            room_id: None,
            sender: UserId::try_from("@carl:example.com").unwrap(),
            state_key: "example.com".to_string(),
            unsigned: None,
        };
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
        serde_json_eq_try_from_raw(event, json);
    }

    #[test]
    fn some_optional_fields_as_some() {
        let event = AliasesEvent {
            content: AliasesEventContent {
                aliases: vec![RoomAliasId::try_from("#room:example.org").unwrap()],
            },
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UInt::try_from(1).unwrap(),
            prev_content: Some(AliasesEventContent {
                aliases: Vec::with_capacity(0),
            }),
            room_id: Some(RoomId::try_from("!n8f893n9:example.com").unwrap()),
            sender: UserId::try_from("@carl:example.com").unwrap(),
            state_key: "example.com".to_string(),
            unsigned: None,
        };
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
        serde_json_eq_try_from_raw(event, json);
    }

    #[test]
    fn all_optional_fields_as_some() {
        let event = AliasesEvent {
            content: AliasesEventContent {
                aliases: vec![RoomAliasId::try_from("#room:example.org").unwrap()],
            },
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UInt::try_from(1).unwrap(),
            prev_content: Some(AliasesEventContent {
                aliases: Vec::with_capacity(0),
            }),
            room_id: Some(RoomId::try_from("!n8f893n9:example.com").unwrap()),
            sender: UserId::try_from("@carl:example.com").unwrap(),
            state_key: "example.com".to_string(),
            unsigned: Some(serde_json::from_str::<Value>(r#"{"foo":"bar"}"#).unwrap()),
        };
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
                "foo": "bar"
            },
            "type": "m.room.aliases"
        });
        serde_json_eq_try_from_raw(event, json);
    }
}

mod extra_fields {
    use super::*;

    ruma_event! {
        /// A redaction of an event.
        RedactionEvent {
            kind: RoomEvent,
            event_type: RoomRedaction,
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
        let event = RedactionEvent {
            content: RedactionEventContent { reason: None },
            redacts: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UInt::try_from(1).unwrap(),
            room_id: Some(RoomId::try_from("!n8f893n9:example.com").unwrap()),
            sender: UserId::try_from("@carl:example.com").unwrap(),
            unsigned: Some(serde_json::from_str::<Value>(r#"{"foo":"bar"}"#).unwrap()),
        };
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
                "foo": "bar"
            },
            "type": "m.room.redaction"
        });
        serde_json_eq_try_from_raw(event, json);
    }
}

mod type_alias {
    use super::*;

    ruma_event! {
        /// Informs the client about the rooms that are considered direct by a user.
        DirectEvent {
            kind: Event,
            event_type: Direct,
            content_type_alias: {
                /// The payload of a `DirectEvent`.
                ///
                /// A mapping of `UserId`'s to a collection of `RoomId`'s which are considered
                /// *direct* for that particular user.
                HashMap<ruma_identifiers::UserId, Vec<ruma_identifiers::RoomId>>
            }
        }
    }

    #[test]
    fn alias_is_not_empty() {
        let content = vec![(
            UserId::try_from("@bob:example.com").unwrap(),
            vec![RoomId::try_from("!n8f893n9:example.com").unwrap()],
        )]
        .into_iter()
        .collect();

        let event = DirectEvent { content };
        let json = json!({
            "content": {
                "@bob:example.com": ["!n8f893n9:example.com"]
            },
            "type": "m.direct"
        });
        serde_json_eq_try_from_raw(event, json);
    }

    #[test]
    fn alias_empty() {
        let content = Default::default();
        let event = DirectEvent { content };
        let json = json!({
            "content": {},
            "type": "m.direct"
        });
        serde_json_eq_try_from_raw(event, json);
    }
}
