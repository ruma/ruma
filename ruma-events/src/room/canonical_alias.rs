//! Types for the *m.room.canonical_alias* event.

use ruma_events_macros::StateEventContent;
use ruma_identifiers::RoomAliasId;
use serde::{Deserialize, Serialize};

use crate::StateEvent;

/// Informs the room as to which alias is the canonical one.
pub type CanonicalAliasEvent = StateEvent<CanonicalAliasEventContent>;

/// The payload for `CanonicalAliasEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, StateEventContent)]
#[ruma_event(type = "m.room.canonical_alias")]
pub struct CanonicalAliasEventContent {
    /// The canonical alias.
    ///
    /// Rooms with `alias: None` should be treated the same as a room
    /// with no canonical alias.
    #[serde(
        default,
        deserialize_with = "ruma_serde::empty_string_as_none",
        skip_serializing_if = "Option::is_none"
    )]
    pub alias: Option<RoomAliasId>,

    /// List of alternative aliases to the room.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alt_aliases: Vec<RoomAliasId>,
}

#[cfg(test)]
mod tests {
    use std::{
        convert::TryFrom,
        time::{Duration, UNIX_EPOCH},
    };

    use ruma_identifiers::{EventId, RoomAliasId, RoomId, UserId};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::CanonicalAliasEventContent;
    use crate::{EventJson, StateEvent, Unsigned};

    #[test]
    fn serialization_with_optional_fields_as_none() {
        let canonical_alias_event = StateEvent {
            content: CanonicalAliasEventContent {
                alias: Some(RoomAliasId::try_from("#somewhere:localhost").unwrap()),
                alt_aliases: Vec::new(),
            },
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
            prev_content: None,
            room_id: RoomId::try_from("!dummy:example.com").unwrap(),
            sender: UserId::try_from("@carl:example.com").unwrap(),
            state_key: "".into(),
            unsigned: Unsigned::default(),
        };

        let actual = to_json_value(&canonical_alias_event).unwrap();
        let expected = json!({
            "content": {
                "alias": "#somewhere:localhost"
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!dummy:example.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.canonical_alias"
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn absent_field_as_none() {
        let json_data = json!({
            "content": {},
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!dummy:example.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.canonical_alias"
        });

        assert_eq!(
            from_json_value::<EventJson<StateEvent<CanonicalAliasEventContent>>>(json_data)
                .unwrap()
                .deserialize()
                .unwrap()
                .content
                .alias,
            None
        );
    }

    #[test]
    fn null_field_as_none() {
        let json_data = json!({
            "content": {
                "alias": null
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!dummy:example.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.canonical_alias"
        });
        assert_eq!(
            from_json_value::<EventJson<StateEvent<CanonicalAliasEventContent>>>(json_data)
                .unwrap()
                .deserialize()
                .unwrap()
                .content
                .alias,
            None
        );
    }

    #[test]
    fn empty_field_as_none() {
        let json_data = json!({
            "content": {
                "alias": ""
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!dummy:example.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.canonical_alias"
        });
        assert_eq!(
            from_json_value::<EventJson<StateEvent<CanonicalAliasEventContent>>>(json_data)
                .unwrap()
                .deserialize()
                .unwrap()
                .content
                .alias,
            None
        );
    }

    #[test]
    fn nonempty_field_as_some() {
        let alias = Some(RoomAliasId::try_from("#somewhere:localhost").unwrap());
        let json_data = json!({
            "content": {
                "alias": "#somewhere:localhost"
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!dummy:example.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.canonical_alias"
        });
        assert_eq!(
            from_json_value::<EventJson<StateEvent<CanonicalAliasEventContent>>>(json_data)
                .unwrap()
                .deserialize()
                .unwrap()
                .content
                .alias,
            alias
        );
    }
}
