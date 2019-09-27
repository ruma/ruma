//! Types for the *m.room.create* event.

use std::convert::TryFrom;

use ruma_events_macros::ruma_event;
use ruma_identifiers::{EventId, RoomId, RoomVersionId, UserId};
use serde::{Deserialize, Serialize};

use crate::default_true;

ruma_event! {
    /// This is the first event in a room and cannot be changed. It acts as the root of all other
    /// events.
    CreateEvent {
        kind: StateEvent,
        event_type: RoomCreate,
        content: {
            /// The `user_id` of the room creator. This is set by the homeserver.
            pub creator: UserId,

            /// Whether or not this room's data should be transferred to other homeservers.
            #[serde(rename = "m.federate")]
            #[serde(default = "default_true")]
            pub federate: bool,

            /// The version of the room. Defaults to "1" if the key does not exist.
            #[serde(default = "default_room_version_id")]
            pub room_version: RoomVersionId,

            /// A reference to the room this room replaces, if the previous room was upgraded.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub predecessor: Option<PreviousRoom>,
        },
    }
}

/// A reference to an old room replaced during a room version upgrade.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PreviousRoom {
    /// The ID of the old room.
    pub room_id: RoomId,

    /// The event ID of the last known event in the old room.
    pub event_id: EventId,
}

/// Used to default the `room_version` field to room version 1.
fn default_room_version_id() -> RoomVersionId {
    RoomVersionId::try_from("1").unwrap()
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use ruma_identifiers::{RoomVersionId, UserId};

    use super::CreateEventContent;
    use crate::EventResult;

    #[test]
    fn serialization() {
        let content = CreateEventContent {
            creator: UserId::try_from("@carl:example.com").unwrap(),
            federate: true,
            room_version: RoomVersionId::version_4(),
            predecessor: None,
        };

        let json = r#"{"creator":"@carl:example.com","m.federate":true,"room_version":"4"}"#;

        assert_eq!(serde_json::to_string(&content).unwrap(), json);
    }

    #[test]
    fn deserialization() {
        let content = CreateEventContent {
            creator: UserId::try_from("@carl:example.com").unwrap(),
            federate: true,
            room_version: RoomVersionId::version_4(),
            predecessor: None,
        };

        let json = r#"{"creator":"@carl:example.com","m.federate":true,"room_version":"4"}"#;

        assert_eq!(
            serde_json::from_str::<EventResult<CreateEventContent>>(json)
                .unwrap()
                .into_result()
                .unwrap(),
            content
        );
    }
}
