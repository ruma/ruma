//! Types for the [`m.room.canonical_alias`] event.
//!
//! [`m.room.canonical_alias`]: https://spec.matrix.org/v1.2/client-server-api/#mroomcanonical_alias

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::RoomAliasId;

/// The content of an `m.room.canonical_alias` event.
///
/// Informs the room as to which alias is the canonical one.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.canonical_alias", kind = State)]
pub struct RoomCanonicalAliasEventContent {
    /// The canonical alias.
    ///
    /// Rooms with `alias: None` should be treated the same as a room
    /// with no canonical alias.
    #[serde(
        default,
        deserialize_with = "ruma_serde::empty_string_as_none",
        skip_serializing_if = "Option::is_none"
    )]
    pub alias: Option<Box<RoomAliasId>>,

    /// List of alternative aliases to the room.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alt_aliases: Vec<Box<RoomAliasId>>,
}

impl RoomCanonicalAliasEventContent {
    /// Creates an empty `RoomCanonicalAliasEventContent`.
    pub fn new() -> Self {
        Self { alias: None, alt_aliases: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use crate::{event_id, room_alias_id, room_id, user_id, MilliSecondsSinceUnixEpoch};
    use js_int::uint;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::RoomCanonicalAliasEventContent;
    use crate::events::{StateEvent, Unsigned};

    #[test]
    fn serialization_with_optional_fields_as_none() {
        let canonical_alias_event = StateEvent {
            content: RoomCanonicalAliasEventContent {
                alias: Some(room_alias_id!("#somewhere:localhost").to_owned()),
                alt_aliases: Vec::new(),
            },
            event_id: event_id!("$h29iv0s8:example.com").to_owned(),
            origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
            prev_content: None,
            room_id: room_id!("!dummy:example.com").to_owned(),
            sender: user_id!("@carl:example.com").to_owned(),
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
            from_json_value::<StateEvent<RoomCanonicalAliasEventContent>>(json_data)
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
            from_json_value::<StateEvent<RoomCanonicalAliasEventContent>>(json_data)
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
            from_json_value::<StateEvent<RoomCanonicalAliasEventContent>>(json_data)
                .unwrap()
                .content
                .alias,
            None
        );
    }

    #[test]
    fn nonempty_field_as_some() {
        let alias = Some(room_alias_id!("#somewhere:localhost").to_owned());
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
            from_json_value::<StateEvent<RoomCanonicalAliasEventContent>>(json_data)
                .unwrap()
                .content
                .alias,
            alias
        );
    }
}
