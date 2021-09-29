//! Types for the *m.room.name* event.

use ruma_events_macros::EventContent;
use ruma_identifiers::RoomNameBox;
use serde::{Deserialize, Serialize};

/// The content of an `m.room.name` event.
///
/// The room name is a human-friendly string designed to be displayed to the end-user.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "m.room.name", kind = State)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct NameEventContent {
    /// The name of the room.
    #[serde(default, deserialize_with = "ruma_serde::empty_string_as_none")]
    pub name: Option<RoomNameBox>,
}

impl NameEventContent {
    /// Create a new `NameEventContent` with the given name.
    pub fn new(name: Option<RoomNameBox>) -> Self {
        Self { name }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use js_int::{int, uint};
    use matches::assert_matches;
    use ruma_common::MilliSecondsSinceUnixEpoch;
    use ruma_identifiers::{event_id, room_id, user_id, RoomNameBox};
    use ruma_serde::Raw;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::NameEventContent;
    use crate::{StateEvent, Unsigned};

    #[test]
    fn serialization_with_optional_fields_as_none() {
        let name_event = StateEvent {
            content: NameEventContent { name: RoomNameBox::try_from("The room name").ok() },
            event_id: event_id!("$h29iv0s8:example.com"),
            origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
            prev_content: None,
            room_id: room_id!("!n8f893n9:example.com"),
            sender: user_id!("@carl:example.com"),
            state_key: "".into(),
            unsigned: Unsigned::default(),
        };

        let actual = to_json_value(&name_event).unwrap();
        let expected = json!({
            "content": {
                "name": "The room name"
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.name"
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn serialization_with_all_fields() {
        let name_event = StateEvent {
            content: NameEventContent { name: RoomNameBox::try_from("The room name").ok() },
            event_id: event_id!("$h29iv0s8:example.com"),
            origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
            prev_content: Some(NameEventContent {
                name: RoomNameBox::try_from("The old name").ok(),
            }),
            room_id: room_id!("!n8f893n9:example.com"),
            sender: user_id!("@carl:example.com"),
            state_key: "".into(),
            unsigned: Unsigned { age: Some(int!(100)), ..Unsigned::default() },
        };

        let actual = to_json_value(&name_event).unwrap();
        let expected = json!({
            "content": {
                "name": "The room name"
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "prev_content": { "name": "The old name" },
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.name",
            "unsigned": {
                "age": 100
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn absent_field_as_none() {
        let json_data = json!({
            "content": {},
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.name"
        });
        assert_eq!(
            from_json_value::<StateEvent<NameEventContent>>(json_data).unwrap().content.name,
            None
        );
    }

    #[test]
    fn name_fails_validation_when_too_long() {
        // "XXXX .." 256 times
        let long_string: String = "X".repeat(256);
        assert_eq!(long_string.len(), 256);

        let long_content_json = json!({ "name": &long_string });
        let from_raw: Raw<NameEventContent> = from_json_value(long_content_json).unwrap();

        let result = from_raw.deserialize();
        assert!(result.is_err(), "Result should be invalid: {:?}", result);
    }

    #[test]
    fn json_with_empty_name_creates_content_as_none() {
        let long_content_json = json!({ "name": "" });
        let from_raw: Raw<NameEventContent> = from_json_value(long_content_json).unwrap();
        assert_matches!(from_raw.deserialize().unwrap(), NameEventContent { name: None });
    }

    #[test]
    fn new_with_empty_name_creates_content_as_none() {
        assert_matches!(
            NameEventContent::new(RoomNameBox::try_from(String::new()).ok()),
            NameEventContent { name: None }
        );
    }

    #[test]
    fn null_field_as_none() {
        let json_data = json!({
            "content": {
                "name": null
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.name"
        });
        assert_eq!(
            from_json_value::<StateEvent<NameEventContent>>(json_data).unwrap().content.name,
            None
        );
    }

    #[test]
    fn empty_string_as_none() {
        let json_data = json!({
            "content": {
                "name": ""
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.name"
        });
        assert_eq!(
            from_json_value::<StateEvent<NameEventContent>>(json_data).unwrap().content.name,
            None
        );
    }

    #[test]
    fn nonempty_field_as_some() {
        let name = RoomNameBox::try_from("The room name").ok();
        let json_data = json!({
            "content": {
                "name": "The room name"
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.name"
        });

        assert_eq!(
            from_json_value::<StateEvent<NameEventContent>>(json_data).unwrap().content.name,
            name
        );
    }
}
