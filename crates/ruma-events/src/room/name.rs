//! Types for the [`m.room.name`] event.
//!
//! [`m.room.name`]: https://spec.matrix.org/latest/client-server-api/#mroomname

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::EmptyStateKey;

/// The content of an `m.room.name` event.
///
/// The room name is a human-friendly string designed to be displayed to the end-user.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.room.name", kind = State, state_key_type = EmptyStateKey)]
pub struct RoomNameEventContent {
    /// The name of the room.
    pub name: String,
}

impl RoomNameEventContent {
    /// Create a new `RoomNameEventContent` with the given name.
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[cfg(test)]
mod tests {
    use ruma_common::canonical_json::assert_to_canonical_json_eq;
    use serde_json::{from_value as from_json_value, json};

    use super::RoomNameEventContent;
    use crate::OriginalStateEvent;

    #[test]
    fn serialization() {
        let content = RoomNameEventContent { name: "The room name".to_owned() };

        assert_to_canonical_json_eq!(
            content,
            json!({
                "name": "The room name",
            }),
        );
    }

    #[test]
    fn deserialization() {
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
            from_json_value::<OriginalStateEvent<RoomNameEventContent>>(json_data)
                .unwrap()
                .content
                .name,
            "The room name"
        );
    }
}
