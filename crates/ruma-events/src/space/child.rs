//! Types for the [`m.space.child`] event.
//!
//! [`m.space.child`]: https://spec.matrix.org/unstable/client-server-api/#mspacechild

use ruma_events_macros::EventContent;
use ruma_identifiers::ServerName;
use serde::{Deserialize, Serialize};

/// The content of an `m.space.child` event.
///
/// The admins of a space can advertise rooms and subspaces for their space by setting
/// `m.space.child` state events.
///
/// The `state_key` is the ID of a child room or space, and the content must contain a `via` key
/// which gives a list of candidate servers that can be used to join the room.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.space.child", kind = State)]
pub struct SpaceChildEventContent {
    /// List of candidate servers that can be used to join the room.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via: Option<Vec<Box<ServerName>>>,

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested: Option<bool>,
}

impl SpaceChildEventContent {
    /// Creates a new `ChildEventContent`.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use ruma_identifiers::server_name;
    use serde_json::{json, to_value as to_json_value};

    use super::SpaceChildEventContent;

    #[test]
    fn space_child_serialization() {
        let content = SpaceChildEventContent {
            via: Some(vec![server_name!("example.com").to_owned()]),
            order: Some("uwu".to_owned()),
            suggested: Some(false),
        };

        let json = json!({
            "via": ["example.com"],
            "order": "uwu",
            "suggested": false,
        });

        assert_eq!(to_json_value(&content).unwrap(), json);
    }

    #[test]
    fn space_child_empty_serialization() {
        let content = SpaceChildEventContent { via: None, order: None, suggested: None };

        let json = json!({});

        assert_eq!(to_json_value(&content).unwrap(), json);
    }
}
