//! Types for the [`m.room.create`] event.
//!
//! [`m.room.create`]: https://spec.matrix.org/latest/client-server-api/#mroomcreate

use ruma_common::{room::RoomType, OwnedEventId, OwnedRoomId, OwnedUserId, RoomVersionId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as,DefaultOnError};

use crate::{EmptyStateKey, RedactContent, RedactedStateEventContent};

/// The content of an `m.room.create` event.
///
/// This is the first event in a room and cannot be changed.
///
/// It acts as the root of all other events.
#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.create", kind = State, state_key_type = EmptyStateKey, custom_redacted)]
pub struct RoomCreateEventContent {
    /// The `user_id` of the room creator.
    ///
    /// This is set by the homeserver.
    ///
    /// This is required in room versions 1 trough 10, but is removed starting from room version
    /// 11.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[deprecated = "Since Matrix 1.8. This field was removed in Room version 11, clients should use the event's sender instead"]
    pub creator: Option<OwnedUserId>,

    /// Whether or not this room's data should be transferred to other homeservers.
    #[serde(
        rename = "m.federate",
        default = "ruma_common::serde::default_true",
        skip_serializing_if = "ruma_common::serde::is_true"
    )]
    pub federate: bool,

    /// The version of the room.
    ///
    /// Defaults to `RoomVersionId::V1`.
    #[serde(default = "default_room_version_id")]
    pub room_version: RoomVersionId,

    /// A reference to the room this room replaces, if the previous room was upgraded.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub predecessor: Option<PreviousRoom>,

    /// The room type.
    ///
    /// This is currently only used for spaces.
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub room_type: Option<RoomType>,
}

impl RoomCreateEventContent {
    /// Creates a new `RoomCreateEventContent` with the given creator, as required for room versions
    /// 1 through 10.
    pub fn new_v1(creator: OwnedUserId) -> Self {
        #[allow(deprecated)]
        Self {
            creator: Some(creator),
            federate: true,
            room_version: default_room_version_id(),
            predecessor: None,
            room_type: None,
        }
    }

    /// Creates a new `RoomCreateEventContent` with the default values and no creator, as introduced
    /// in room version 11.
    ///
    /// The room version is set to [`RoomVersionId::V11`].
    pub fn new_v11() -> Self {
        #[allow(deprecated)]
        Self {
            creator: None,
            federate: true,
            room_version: RoomVersionId::V11,
            predecessor: None,
            room_type: None,
        }
    }
}

impl RedactContent for RoomCreateEventContent {
    type Redacted = RedactedRoomCreateEventContent;

    fn redact(self, version: &RoomVersionId) -> Self::Redacted {
        #[allow(deprecated)]
        match version {
            RoomVersionId::V1
            | RoomVersionId::V2
            | RoomVersionId::V3
            | RoomVersionId::V4
            | RoomVersionId::V5
            | RoomVersionId::V6
            | RoomVersionId::V7
            | RoomVersionId::V8
            | RoomVersionId::V9
            | RoomVersionId::V10 => Self {
                room_version: default_room_version_id(),
                creator: self.creator,
                ..Self::new_v11()
            },
            _ => self,
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

/// Redacted form of [`RoomCreateEventContent`].
///
/// The redaction rules of this event changed with room version 11:
///
/// - In room versions 1 through 10, the `creator` field was preserved during redaction, starting
///   from room version 11 the field is removed.
/// - In room versions 1 through 10, all the other fields were redacted, starting from room version
///   11 all the fields are preserved.
pub type RedactedRoomCreateEventContent = RoomCreateEventContent;

impl RedactedStateEventContent for RedactedRoomCreateEventContent {
    type StateKey = EmptyStateKey;
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::{owned_user_id, RoomVersionId};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{RoomCreateEventContent, RoomType};

    #[test]
    fn serialization() {
        #[allow(deprecated)]
        let content = RoomCreateEventContent {
            creator: Some(owned_user_id!("@carl:example.com")),
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
        #[allow(deprecated)]
        let content = RoomCreateEventContent {
            creator: Some(owned_user_id!("@carl:example.com")),
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
    #[allow(deprecated)]
    fn deserialization() {
        let json = json!({
            "creator": "@carl:example.com",
            "m.federate": true,
            "room_version": "4"
        });

        let content = from_json_value::<RoomCreateEventContent>(json).unwrap();
        assert_eq!(content.creator.unwrap(), "@carl:example.com");
        assert!(content.federate);
        assert_eq!(content.room_version, RoomVersionId::V4);
        assert_matches!(content.predecessor, None);
        assert_eq!(content.room_type, None);
    }

    #[test]
    #[allow(deprecated)]
    fn space_deserialization() {
        let json = json!({
            "creator": "@carl:example.com",
            "m.federate": true,
            "room_version": "4",
            "type": "m.space"
        });

        let content = from_json_value::<RoomCreateEventContent>(json).unwrap();
        assert_eq!(content.creator.unwrap(), "@carl:example.com");
        assert!(content.federate);
        assert_eq!(content.room_version, RoomVersionId::V4);
        assert_matches!(content.predecessor, None);
        assert_eq!(content.room_type, Some(RoomType::Space));
    }

    #[test]
    #[allow(deprecated)]
    fn deserialize_foundation() {
        let json = json!({
            "creator": "@abuse:matrix.org",
            "predecessor": "!dSMpkVKGgQHlgBDSpo:matrix.org",
            "room_version": "10"
        });

        let content = from_json_value::<RoomCreateEventContent>(json).unwrap();
        print!("{:?}", content);
        assert_eq!(content.creator.unwrap(), "@abuse:matrix.org");
        assert!(content.federate);
        assert_eq!(content.room_version, RoomVersionId::V10);
        assert_eq!(content.room_type, None);
        assert_matches!(content.predecessor, None);
    }

    #[test]
    #[allow(deprecated)]
    fn deserialize_foundation_raw() {
        let json = r#"
            {
                "creator": "@abuse:matrix.org",
                "predecessor": "!dSMpkVKGgQHlgBDSpo:matrix.org",
                "room_version": "10"
            }
        "#;

        let content = serde_json::from_str::<RoomCreateEventContent>(json).unwrap();
        assert_eq!(content.creator.unwrap(), "@abuse:matrix.org");
        assert!(content.federate);
        assert_eq!(content.room_version, RoomVersionId::V10);
        assert_eq!(content.room_type, None);
        assert_matches!(content.predecessor, None);
    }
}
