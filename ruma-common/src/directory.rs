//! Common types for room directory endpoints

use std::fmt;

use js_int::UInt;
use ruma_identifiers::{RoomAliasId, RoomId};
use ruma_serde::Outgoing;
use serde::{
    de::{Error, MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_json::Value as JsonValue;

/// A chunk of a room list response, describing one room.
///
/// To create an instance of this type, first create a `PublicRoomsChunkInit` and convert it via
/// `PublicRoomsChunk::from` / `.into()`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct PublicRoomsChunk {
    /// Aliases of the room.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<RoomAliasId>,

    /// The canonical alias of the room, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_alias: Option<RoomAliasId>,

    /// The name of the room, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The number of members joined to the room.
    pub num_joined_members: UInt,

    /// The ID of the room.
    pub room_id: RoomId,

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

/// Initial set of mandatory fields of `PublicRoomsChunk`.
///
/// This struct will not be updated even if additional fields are added to `PublicRoomsChunk` in a
/// new (non-breaking) release of the Matrix specification.
#[derive(Debug)]
pub struct PublicRoomsChunkInit {
    /// The number of members joined to the room.
    pub num_joined_members: UInt,

    /// The ID of the room.
    pub room_id: RoomId,

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
            aliases: Vec::new(),
            canonical_alias: None,
            name: None,
            num_joined_members,
            room_id,
            topic: None,
            world_readable,
            guest_can_join,
            avatar_url: None,
        }
    }
}

/// A filter for public rooms lists
#[derive(Clone, Debug, Default, Outgoing, Serialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, Outgoing)]
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

impl<'a> Serialize for RoomNetwork<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state;
        match self {
            Self::Matrix => {
                state = serializer.serialize_struct("RoomNetwork", 0)?;
            }
            Self::All => {
                state = serializer.serialize_struct("RoomNetwork", 1)?;
                state.serialize_field("include_all_networks", &true)?;
            }
            Self::ThirdParty(network) => {
                state = serializer.serialize_struct("RoomNetwork", 1)?;
                state.serialize_field("third_party_instance_id", network)?;
            }
        }
        state.end()
    }
}

impl<'de> Deserialize<'de> for IncomingRoomNetwork {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(RoomNetworkVisitor)
    }
}

struct RoomNetworkVisitor;
impl<'de> Visitor<'de> for RoomNetworkVisitor {
    type Value = IncomingRoomNetwork;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Network selection")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut include_all_networks = false;
        let mut third_party_instance_id = None;
        while let Some((key, value)) = access.next_entry::<String, JsonValue>()? {
            match key.as_str() {
                "include_all_networks" => {
                    include_all_networks = match value.as_bool() {
                        Some(b) => b,
                        _ => false,
                    }
                }
                "third_party_instance_id" => {
                    third_party_instance_id = value.as_str().map(|v| v.to_owned())
                }
                _ => {}
            };
        }

        if include_all_networks {
            if third_party_instance_id.is_none() {
                Ok(IncomingRoomNetwork::All)
            } else {
                Err(M::Error::custom(
                    "`include_all_networks = true` and `third_party_instance_id` are mutually exclusive.",
                ))
            }
        } else {
            Ok(match third_party_instance_id {
                Some(network) => IncomingRoomNetwork::ThirdParty(network),
                None => IncomingRoomNetwork::Matrix,
            })
        }
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
