//! Types for the [`m.space.parent`] event.
//!
//! [`m.space.parent`]: https://spec.matrix.org/v1.2/client-server-api/#mspaceparent

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{OwnedRoomId, OwnedServerName};

/// The content of an `m.space.parent` event.
///
/// Rooms can claim parents via the `m.space.parent` state event.
///
/// Similar to `m.space.child`, the `state_key` is the ID of the parent space, and the content must
/// contain a `via` key which gives a list of candidate servers that can be used to join the
/// parent.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.space.parent", kind = State, state_key_type = OwnedRoomId)]
pub struct SpaceParentEventContent {
    /// List of candidate servers that can be used to join the room.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via: Option<Vec<OwnedServerName>>,

    /// Determines whether this is the main parent for the space.
    ///
    /// When a user joins a room with a canonical parent, clients may switch to view the room in
    /// the context of that space, peeking into it in order to find other rooms and group them
    /// together. In practice, well behaved rooms should only have one `canonical` parent, but
    /// given this is not enforced: if multiple are present the client should select the one with
    /// the lowest room ID, as determined via a lexicographic ordering of the Unicode code-points.
    pub canonical: bool,
}

impl SpaceParentEventContent {
    /// Creates a new `ParentEventContent` with the given canonical flag.
    pub fn new(canonical: bool) -> Self {
        Self { via: None, canonical }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{json, to_value as to_json_value};

    use super::SpaceParentEventContent;
    use crate::server_name;

    #[test]
    fn space_parent_serialization() {
        let content = SpaceParentEventContent {
            via: Some(vec![server_name!("example.com").to_owned()]),
            canonical: true,
        };

        let json = json!({
            "via": ["example.com"],
            "canonical": true,
        });

        assert_eq!(to_json_value(&content).unwrap(), json);
    }

    #[test]
    fn space_parent_empty_serialization() {
        let content = SpaceParentEventContent { via: None, canonical: true };

        let json = json!({
            "canonical": true,
        });

        assert_eq!(to_json_value(&content).unwrap(), json);
    }
}
