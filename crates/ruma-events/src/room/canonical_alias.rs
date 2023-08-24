//! Types for the [`m.room.canonical_alias`] event.
//!
//! [`m.room.canonical_alias`]: https://spec.matrix.org/latest/client-server-api/#mroomcanonical_alias

use ruma_common::OwnedRoomAliasId;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::EmptyStateKey;

/// The content of an `m.room.canonical_alias` event.
///
/// Informs the room as to which alias is the canonical one.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.canonical_alias", kind = State, state_key_type = EmptyStateKey)]
pub struct RoomCanonicalAliasEventContent {
    /// The canonical alias.
    ///
    /// Rooms with `alias: None` should be treated the same as a room
    /// with no canonical alias.
    #[serde(
        default,
        deserialize_with = "ruma_common::serde::empty_string_as_none",
        skip_serializing_if = "Option::is_none"
    )]
    pub alias: Option<OwnedRoomAliasId>,

    /// List of alternative aliases to the room.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alt_aliases: Vec<OwnedRoomAliasId>,
}

impl RoomCanonicalAliasEventContent {
    /// Creates an empty `RoomCanonicalAliasEventContent`.
    pub fn new() -> Self {
        Self { alias: None, alt_aliases: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use ruma_common::owned_room_alias_id;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::RoomCanonicalAliasEventContent;
    use crate::OriginalStateEvent;

    #[test]
    fn serialization_with_optional_fields_as_none() {
        let content = RoomCanonicalAliasEventContent {
            alias: Some(owned_room_alias_id!("#somewhere:localhost")),
            alt_aliases: Vec::new(),
        };

        let actual = to_json_value(&content).unwrap();
        let expected = json!({
            "alias": "#somewhere:localhost",
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
            from_json_value::<OriginalStateEvent<RoomCanonicalAliasEventContent>>(json_data)
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
            from_json_value::<OriginalStateEvent<RoomCanonicalAliasEventContent>>(json_data)
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
            from_json_value::<OriginalStateEvent<RoomCanonicalAliasEventContent>>(json_data)
                .unwrap()
                .content
                .alias,
            None
        );
    }

    #[test]
    fn nonempty_field_as_some() {
        let alias = Some(owned_room_alias_id!("#somewhere:localhost"));
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
            from_json_value::<OriginalStateEvent<RoomCanonicalAliasEventContent>>(json_data)
                .unwrap()
                .content
                .alias,
            alias
        );
    }
}
