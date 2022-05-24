//! Types for the [`m.room.create`] event.
//!
//! [`m.room.create`]: https://spec.matrix.org/v1.2/client-server-api/#mroomcreate

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{
    events::EmptyStateKey, room::RoomType, OwnedEventId, OwnedRoomId, OwnedUserId, RoomVersionId,
};

/// The content of an `m.room.create` event.
///
/// This is the first event in a room and cannot be changed.
///
/// It acts as the root of all other events.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.create", kind = State, state_key_type = EmptyStateKey)]
pub struct RoomCreateEventContent {
    /// The `user_id` of the room creator.
    ///
    /// This is set by the homeserver.
    #[ruma_event(skip_redaction)]
    pub creator: OwnedUserId,

    /// Whether or not this room's data should be transferred to other homeservers.
    #[serde(
        rename = "m.federate",
        default = "crate::serde::default_true",
        skip_serializing_if = "crate::serde::is_true"
    )]
    pub federate: bool,

    /// The version of the room.
    ///
    /// Defaults to `RoomVersionId::V1`.
    #[serde(default = "default_room_version_id")]
    pub room_version: RoomVersionId,

    /// A reference to the room this room replaces, if the previous room was upgraded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub predecessor: Option<PreviousRoom>,

    /// The room type.
    ///
    /// This is currently only used for spaces.
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub room_type: Option<RoomType>,
}

impl RoomCreateEventContent {
    /// Creates a new `RoomCreateEventContent` with the given creator.
    pub fn new(creator: OwnedUserId) -> Self {
        Self {
            creator,
            federate: true,
            room_version: default_room_version_id(),
            predecessor: None,
            room_type: None,
        }
    }
}

/// A reference to an old room replaced during a room version upgrade.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct PreviousRoom {
    /// The ID of the old room.
    pub room_id: OwnedRoomId,

    /// The event ID of the last known event in the old room.
    pub event_id: OwnedEventId,
}

impl PreviousRoom {
    /// Creates a new `PreviousRoom` from the given room and event IDs.
    pub fn new(room_id: OwnedRoomId, event_id: OwnedEventId) -> Self {
        Self { room_id, event_id }
    }
}

/// Used to default the `room_version` field to room version 1.
fn default_room_version_id() -> RoomVersionId {
    RoomVersionId::V1
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{RoomCreateEventContent, RoomType};
    use crate::{user_id, RoomVersionId};

    #[test]
    fn serialization() {
        let content = RoomCreateEventContent {
            creator: user_id!("@carl:example.com").to_owned(),
            federate: false,
            room_version: RoomVersionId::V4,
            predecessor: None,
            room_type: None,
        };

        let json = json!({
            "creator": "@carl:example.com",
            "m.federate": false,
            "room_version": "4"
        });

        assert_eq!(to_json_value(&content).unwrap(), json);
    }

    #[test]
    fn space_serialization() {
        let content = RoomCreateEventContent {
            creator: user_id!("@carl:example.com").to_owned(),
            federate: false,
            room_version: RoomVersionId::V4,
            predecessor: None,
            room_type: Some(RoomType::Space),
        };

        let json = json!({
            "creator": "@carl:example.com",
            "m.federate": false,
            "room_version": "4",
            "type": "m.space"
        });

        assert_eq!(to_json_value(&content).unwrap(), json);
    }

    #[test]
    fn deserialization() {
        let json = json!({
            "creator": "@carl:example.com",
            "m.federate": true,
            "room_version": "4"
        });

        assert_matches!(
            from_json_value::<RoomCreateEventContent>(json).unwrap(),
            RoomCreateEventContent {
                creator,
                federate: true,
                room_version: RoomVersionId::V4,
                predecessor: None,
                room_type: None,
            } if creator == "@carl:example.com"
        );
    }

    #[test]
    fn space_deserialization() {
        let json = json!({
            "creator": "@carl:example.com",
            "m.federate": true,
            "room_version": "4",
            "type": "m.space"
        });

        assert_matches!(
            from_json_value::<RoomCreateEventContent>(json).unwrap(),
            RoomCreateEventContent {
                creator,
                federate: true,
                room_version: RoomVersionId::V4,
                predecessor: None,
                room_type
            } if creator == "@carl:example.com" && room_type == Some(RoomType::Space)
        );
    }
}
