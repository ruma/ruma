//! Types for the [`m.space.child`] event.
//!
//! [`m.space.child`]: https://spec.matrix.org/latest/client-server-api/#mspacechild

use ruma_common::{MilliSecondsSinceUnixEpoch, OwnedRoomId, OwnedServerName, OwnedUserId};
use ruma_macros::{Event, EventContent};
use serde::{Deserialize, Serialize};

/// The content of an `m.space.child` event.
///
/// The admins of a space can advertise rooms and subspaces for their space by setting
/// `m.space.child` state events.
///
/// The `state_key` is the ID of a child room or space, and the content must contain a `via` key
/// which gives a list of candidate servers that can be used to join the room.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.space.child", kind = State, state_key_type = OwnedRoomId)]
pub struct SpaceChildEventContent {
    /// List of candidate servers that can be used to join the room.
    pub via: Vec<OwnedServerName>,

    /// Provide a default ordering of siblings in the room list.
    ///
    /// Rooms are sorted based on a lexicographic ordering of the Unicode codepoints of the
    /// characters in `order` values. Rooms with no `order` come last, in ascending numeric order
    /// of the origin_server_ts of their m.room.create events, or ascending lexicographic order of
    /// their room_ids in case of equal `origin_server_ts`. `order`s which are not strings, or do
    /// not consist solely of ascii characters in the range `\x20` (space) to `\x7E` (`~`), or
    /// consist of more than 50 characters, are forbidden and the field should be ignored if
    /// received.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,

    /// Space admins can mark particular children of a space as "suggested".
    ///
    /// This mainly serves as a hint to clients that that they can be displayed differently, for
    /// example by showing them eagerly in the room list. A child which is missing the `suggested`
    /// property is treated identically to a child with `"suggested": false`. A suggested child may
    /// be a room or a subspace.
    ///
    /// Defaults to `false`.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    pub suggested: bool,
}

impl SpaceChildEventContent {
    /// Creates a new `SpaceChildEventContent` with the given routing servers.
    pub fn new(via: Vec<OwnedServerName>) -> Self {
        Self { via, order: None, suggested: false }
    }
}

/// An `m.space.child` event represented as a Stripped State Event with an added `origin_server_ts`
/// key.
#[derive(Clone, Debug, Event)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct HierarchySpaceChildEvent {
    /// The content of the space child event.
    pub content: SpaceChildEventContent,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,

    /// The room ID of the child.
    pub state_key: OwnedRoomId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,
}

#[cfg(test)]
mod tests {
    use js_int::uint;
    use ruma_common::{server_name, MilliSecondsSinceUnixEpoch};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{HierarchySpaceChildEvent, SpaceChildEventContent};

    #[test]
    fn space_child_serialization() {
        let content = SpaceChildEventContent {
            via: vec![server_name!("example.com").to_owned()],
            order: Some("uwu".to_owned()),
            suggested: false,
        };

        let json = json!({
            "via": ["example.com"],
            "order": "uwu",
        });

        assert_eq!(to_json_value(&content).unwrap(), json);
    }

    #[test]
    fn space_child_empty_serialization() {
        let content = SpaceChildEventContent { via: vec![], order: None, suggested: false };

        let json = json!({ "via": [] });

        assert_eq!(to_json_value(&content).unwrap(), json);
    }

    #[test]
    fn hierarchy_space_child_deserialization() {
        let json = json!({
            "content": {
                "via": [
                    "example.org"
                ]
            },
            "origin_server_ts": 1_629_413_349,
            "sender": "@alice:example.org",
            "state_key": "!a:example.org",
            "type": "m.space.child"
        });

        let ev = from_json_value::<HierarchySpaceChildEvent>(json).unwrap();
        assert_eq!(ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1_629_413_349)));
        assert_eq!(ev.sender, "@alice:example.org");
        assert_eq!(ev.state_key, "!a:example.org");
        assert_eq!(ev.content.via, ["example.org"]);
        assert_eq!(ev.content.order, None);
        assert!(!ev.content.suggested);
    }
}
