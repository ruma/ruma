//! Types for the *m.room.create* event.

use ruma_events_macros::EventContent;
use ruma_identifiers::{EventId, RoomId, RoomVersionId, UserId};
use ruma_serde::StringEnum;
use serde::{Deserialize, Serialize};

use crate::StateEvent;

/// This is the first event in a room and cannot be changed. It acts as the root of all other
/// events.
pub type CreateEvent = StateEvent<CreateEventContent>;

/// The payload for `CreateEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.create", kind = State)]
pub struct CreateEventContent {
    /// The `user_id` of the room creator. This is set by the homeserver.
    #[ruma_event(skip_redaction)]
    pub creator: UserId,

    /// Whether or not this room's data should be transferred to other homeservers.
    #[serde(
        rename = "m.federate",
        default = "ruma_serde::default_true",
        skip_serializing_if = "ruma_serde::is_true"
    )]
    pub federate: bool,

    /// The version of the room. Defaults to "1" if the key does not exist.
    #[serde(default = "default_room_version_id")]
    pub room_version: RoomVersionId,

    /// A reference to the room this room replaces, if the previous room was upgraded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub predecessor: Option<PreviousRoom>,

    /// The room type.
    ///
    /// This is currently only used for spaces.
    #[cfg(feature = "unstable-pre-spec")]
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub room_type: Option<RoomType>,
}

impl CreateEventContent {
    /// Creates a new `CreateEventContent` with the given creator.
    pub fn new(creator: UserId) -> Self {
        Self {
            creator,
            federate: true,
            room_version: default_room_version_id(),
            predecessor: None,
            #[cfg(feature = "unstable-pre-spec")]
            room_type: None,
        }
    }
}

/// An enum of possible room types.
///
/// This type can hold an arbitrary string. To check for formats that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[non_exhaustive]
pub enum RoomType {
    /// Defines the room as a space.
    #[ruma_enum(rename = "m.space")]
    Space,
    /// Defines the room as a custom type.
    #[doc(hidden)]
    _Custom(String),
}

impl RoomType {
    /// Creates a string slice from this `RoomType`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

/// A reference to an old room replaced during a room version upgrade.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct PreviousRoom {
    /// The ID of the old room.
    pub room_id: RoomId,

    /// The event ID of the last known event in the old room.
    pub event_id: EventId,
}

impl PreviousRoom {
    /// Creates a new `PreviousRoom` from the given room and event IDs.
    pub fn new(room_id: RoomId, event_id: EventId) -> Self {
        Self { room_id, event_id }
    }
}

/// Used to default the `room_version` field to room version 1.
fn default_room_version_id() -> RoomVersionId {
    RoomVersionId::Version1
}

#[cfg(test)]
mod tests {
    use matches::assert_matches;
    use ruma_identifiers::{user_id, RoomVersionId};
    use ruma_serde::Raw;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::CreateEventContent;

    #[cfg(feature = "unstable-pre-spec")]
    use super::RoomType;

    #[test]
    fn serialization() {
        let content = CreateEventContent {
            creator: user_id!("@carl:example.com"),
            federate: false,
            room_version: RoomVersionId::Version4,
            predecessor: None,
            #[cfg(feature = "unstable-pre-spec")]
            room_type: None,
        };

        let json = json!({
            "creator": "@carl:example.com",
            "m.federate": false,
            "room_version": "4"
        });

        assert_eq!(to_json_value(&content).unwrap(), json);
    }

    #[cfg(feature = "unstable-pre-spec")]
    #[test]
    fn space_serialization() {
        let content = CreateEventContent {
            creator: user_id!("@carl:example.com"),
            federate: false,
            room_version: RoomVersionId::Version4,
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
            from_json_value::<Raw<CreateEventContent>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            CreateEventContent {
                creator,
                federate: true,
                room_version: RoomVersionId::Version4,
                predecessor: None,
                #[cfg(feature = "unstable-pre-spec")]
                room_type: None,
            } if creator == "@carl:example.com"
        );
    }

    #[cfg(feature = "unstable-pre-spec")]
    #[test]
    fn space_deserialization() {
        let json = json!({
            "creator": "@carl:example.com",
            "m.federate": true,
            "room_version": "4",
            "type": "m.space"
        });

        assert_matches!(
            from_json_value::<Raw<CreateEventContent>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            CreateEventContent {
                creator,
                federate: true,
                room_version: RoomVersionId::Version4,
                predecessor: None,
                room_type
            } if creator == "@carl:example.com" && room_type == Some(RoomType::Space)
        );
    }
}
