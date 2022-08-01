//! Common types for room directory endpoints.

use crate::{
    serde::{Incoming, StringEnum},
    OwnedRoomAliasId, OwnedRoomId,
};
use js_int::UInt;
use serde::{Deserialize, Serialize};

mod room_network_serde;

use crate::{OwnedMxcUri, PrivOwnedStr};

/// A chunk of a room list response, describing one room.
///
/// To create an instance of this type, first create a `PublicRoomsChunkInit` and convert it via
/// `PublicRoomsChunk::from` / `.into()`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct PublicRoomsChunk {
    /// The canonical alias of the room, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "compat",
        serde(default, deserialize_with = "crate::serde::empty_string_as_none")
    )]
    pub canonical_alias: Option<OwnedRoomAliasId>,

    /// The name of the room, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The number of members joined to the room.
    pub num_joined_members: UInt,

    /// The ID of the room.
    pub room_id: OwnedRoomId,

    /// The topic of the room, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,

    /// Whether the room may be viewed by guest users without joining.
    pub world_readable: bool,

    /// Whether guest users may join the room and participate in it.
    ///
    /// If they can, they will be subject to ordinary power level rules like any other user.
    pub guest_can_join: bool,

    /// The URL for the room's avatar, if one is set.
    ///
    /// If you activate the `compat` feature, this field being an empty string in JSON will result
    /// in `None` here during deserialization.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "compat",
        serde(default, deserialize_with = "crate::serde::empty_string_as_none")
    )]
    pub avatar_url: Option<OwnedMxcUri>,

    /// The join rule of the room.
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub join_rule: PublicRoomJoinRule,
}

/// Initial set of mandatory fields of `PublicRoomsChunk`.
///
/// This struct will not be updated even if additional fields are added to `PublicRoomsChunk` in a
/// new (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct PublicRoomsChunkInit {
    /// The number of members joined to the room.
    pub num_joined_members: UInt,

    /// The ID of the room.
    pub room_id: OwnedRoomId,

    /// Whether the room may be viewed by guest users without joining.
    pub world_readable: bool,

    /// Whether guest users may join the room and participate in it.
    ///
    /// If they can, they will be subject to ordinary power level rules like any other user.
    pub guest_can_join: bool,
}

impl From<PublicRoomsChunkInit> for PublicRoomsChunk {
    fn from(init: PublicRoomsChunkInit) -> Self {
        let PublicRoomsChunkInit { num_joined_members, room_id, world_readable, guest_can_join } =
            init;

        Self {
            canonical_alias: None,
            name: None,
            num_joined_members,
            room_id,
            topic: None,
            world_readable,
            guest_can_join,
            avatar_url: None,
            join_rule: PublicRoomJoinRule::default(),
        }
    }
}

/// A filter for public rooms lists
#[derive(Clone, Debug, Default, Incoming, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[incoming_derive(Default)]
pub struct Filter<'a> {
    /// A string to search for in the room metadata, e.g. name, topic, canonical alias etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generic_search_term: Option<&'a str>,
}

impl Filter<'_> {
    /// Creates an empty `Filter`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns `true` if the filter is empty.
    pub fn is_empty(&self) -> bool {
        self.generic_search_term.is_none()
    }
}

/// Information about which networks/protocols from application services on the
/// homeserver from which to request rooms.
#[derive(Clone, Debug, PartialEq, Eq, Incoming)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[incoming_derive(Clone, PartialEq, Eq, !Deserialize)]
pub enum RoomNetwork<'a> {
    /// Return rooms from the Matrix network.
    Matrix,

    /// Return rooms from all the networks/protocols the homeserver knows about.
    All,

    /// Return rooms from a specific third party network/protocol.
    ThirdParty(&'a str),
}

impl<'a> Default for RoomNetwork<'a> {
    fn default() -> Self {
        RoomNetwork::Matrix
    }
}

/// The rule used for users wishing to join a public room.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum PublicRoomJoinRule {
    /// Users can request an invite to the room.
    Knock,

    /// Anyone can join the room without any prior action.
    Public,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl Default for PublicRoomJoinRule {
    fn default() -> Self {
        Self::Public
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{IncomingRoomNetwork, RoomNetwork};

    #[test]
    fn serialize_matrix_network_only() {
        let json = json!({});
        assert_eq!(to_json_value(RoomNetwork::Matrix).unwrap(), json);
    }

    #[test]
    fn deserialize_matrix_network_only() {
        let json = json!({ "include_all_networks": false });
        assert_eq!(
            from_json_value::<IncomingRoomNetwork>(json).unwrap(),
            IncomingRoomNetwork::Matrix
        );
    }

    #[test]
    fn serialize_default_network_is_empty() {
        let json = json!({});
        assert_eq!(to_json_value(RoomNetwork::default()).unwrap(), json);
    }

    #[test]
    fn deserialize_empty_network_is_default() {
        let json = json!({});
        assert_eq!(
            from_json_value::<IncomingRoomNetwork>(json).unwrap(),
            IncomingRoomNetwork::Matrix
        );
    }

    #[test]
    fn serialize_include_all_networks() {
        let json = json!({ "include_all_networks": true });
        assert_eq!(to_json_value(RoomNetwork::All).unwrap(), json);
    }

    #[test]
    fn deserialize_include_all_networks() {
        let json = json!({ "include_all_networks": true });
        assert_eq!(from_json_value::<IncomingRoomNetwork>(json).unwrap(), IncomingRoomNetwork::All);
    }

    #[test]
    fn serialize_third_party_network() {
        let json = json!({ "third_party_instance_id": "freenode" });
        assert_eq!(to_json_value(RoomNetwork::ThirdParty("freenode")).unwrap(), json);
    }

    #[test]
    fn deserialize_third_party_network() {
        let json = json!({ "third_party_instance_id": "freenode" });
        assert_eq!(
            from_json_value::<IncomingRoomNetwork>(json).unwrap(),
            IncomingRoomNetwork::ThirdParty("freenode".into())
        );
    }

    #[test]
    fn deserialize_include_all_networks_and_third_party_exclusivity() {
        let json = json!({ "include_all_networks": true, "third_party_instance_id": "freenode" });
        assert_eq!(
            from_json_value::<IncomingRoomNetwork>(json).unwrap_err().to_string().as_str(),
            "`include_all_networks = true` and `third_party_instance_id` are mutually exclusive."
        );
    }
}
