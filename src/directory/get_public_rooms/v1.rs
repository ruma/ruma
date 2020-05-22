//! [GET /_matrix/federation/v1/publicRooms](https://matrix.org/docs/spec/server_server/r0.1.3#get-matrix-federation-v1-publicrooms)

use std::fmt;

use js_int::UInt;
use ruma_api::ruma_api;
use ruma_identifiers::{RoomAliasId, RoomId};
use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};

ruma_api! {
    metadata {
        description: "Gets all the public rooms for the homeserver.",
        method: GET,
        name: "get_public_rooms",
        path: "/_matrix/federation/v1/publicRooms",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The maximum number of rooms to return. Default is no limit.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub limit: Option<UInt>,
        /// Pagination token from a previous request.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub since: Option<String>,
        /// Network to fetch the public room lists from.
        #[serde(flatten, skip_serializing_if = "ruma_serde::is_default")]
        #[ruma_api(query)]
        pub room_network: RoomNetwork,
    }

    response {
        /// A paginated chunk of public rooms.
        pub chunk: Vec<PublicRoomsChunk>,
        /// A pagination token for the response.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub next_batch: Option<String>,
        /// A pagination token that allows fetching previous results.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub prev_batch: Option<String>,
        /// An estimate on the total number of public rooms, if the server has an estimate.
        pub total_room_count_estimate: Option<UInt>,
    }
}

/// A chunk of a room list response, describing one room
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublicRoomsChunk {
    /// Aliases of the room.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<RoomAliasId>,
    /// The canonical alias of the room, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_alias: Option<String>,
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

/// Information about which networks/protocols from application services on the
/// homeserver from which to request rooms.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RoomNetwork {
    /// Return rooms from the Matrix network.
    Matrix,
    /// Return rooms from all the networks/protocols the homeserver knows about.
    All,
    /// Return rooms from a specific third party network/protocol.
    ThirdParty(String),
}

impl Default for RoomNetwork {
    fn default() -> Self {
        RoomNetwork::Matrix
    }
}

impl Serialize for RoomNetwork {
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

impl<'de> Deserialize<'de> for RoomNetwork {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(RoomNetworkVisitor)
    }
}

struct RoomNetworkVisitor;
impl<'de> Visitor<'de> for RoomNetworkVisitor {
    type Value = RoomNetwork;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Network selection")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut include_all_networks = false;
        let mut third_party_instance_id = None;
        while let Some((key, value)) = access.next_entry::<String, serde_json::Value>()? {
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
                Ok(RoomNetwork::All)
            } else {
                Err(M::Error::custom(
                    "`include_all_networks = true` and `third_party_instance_id` are mutually exclusive.",
                ))
            }
        } else {
            Ok(match third_party_instance_id {
                Some(network) => RoomNetwork::ThirdParty(network),
                None => RoomNetwork::Matrix,
            })
        }
    }
}
