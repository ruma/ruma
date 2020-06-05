//! [POST /_matrix/client/r0/publicRooms](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-publicrooms)

use std::fmt;

use js_int::UInt;
use ruma_api::ruma_api;
use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};

use serde_json::Value as JsonValue;

use super::PublicRoomsChunk;

ruma_api! {
    metadata {
        description: "Get the list of rooms in this homeserver's public directory.",
        method: POST,
        name: "get_public_rooms_filtered",
        path: "/_matrix/client/r0/publicRooms",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The server to fetch the public room lists from.
        ///
        /// `None` means the server this request is sent to.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub server: Option<String>,

        /// Limit for the number of results to return.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub limit: Option<UInt>,

        /// Pagination token from a previous request.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub since: Option<String>,

        /// Filter to apply to the results.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub filter: Option<Filter>,

        /// Network to fetch the public room lists from.
        #[serde(flatten, skip_serializing_if = "ruma_serde::is_default")]
        pub room_network: RoomNetwork,
    }

    response {
        /// A paginated chunk of public rooms.
        pub chunk: Vec<PublicRoomsChunk>,

        /// A pagination token for the response.
        pub next_batch: Option<String>,

        /// A pagination token that allows fetching previous results.
        pub prev_batch: Option<String>,

        /// An estimate on the total number of public rooms, if the server has an estimate.
        pub total_room_count_estimate: Option<UInt>,
    }

    error: crate::Error
}

/// A filter for public rooms lists
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Filter {
    /// A string to search for in the room metadata, e.g. name, topic, canonical alias etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generic_search_term: Option<String>,
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

#[cfg(test)]
mod tests {
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::RoomNetwork;

    #[test]
    fn test_serialize_matrix_network_only() {
        let json = json!({});
        assert_eq!(to_json_value(RoomNetwork::Matrix).unwrap(), json);
    }

    #[test]
    fn test_deserialize_matrix_network_only() {
        let json = json!({ "include_all_networks": false });
        assert_eq!(
            from_json_value::<RoomNetwork>(json).unwrap(),
            RoomNetwork::Matrix
        );
    }

    #[test]
    fn test_serialize_default_network_is_empty() {
        let json = json!({});
        assert_eq!(to_json_value(RoomNetwork::default()).unwrap(), json);
    }

    #[test]
    fn test_deserialize_empty_network_is_default() {
        let json = json!({});
        assert_eq!(
            from_json_value::<RoomNetwork>(json).unwrap(),
            RoomNetwork::default()
        );
    }

    #[test]
    fn test_serialize_include_all_networks() {
        let json = json!({ "include_all_networks": true });
        assert_eq!(to_json_value(RoomNetwork::All).unwrap(), json);
    }

    #[test]
    fn test_deserialize_include_all_networks() {
        let json = json!({ "include_all_networks": true });
        assert_eq!(
            from_json_value::<RoomNetwork>(json).unwrap(),
            RoomNetwork::All
        );
    }

    #[test]
    fn test_serialize_third_party_network() {
        let json = json!({ "third_party_instance_id": "freenode" });
        assert_eq!(
            to_json_value(RoomNetwork::ThirdParty("freenode".to_string())).unwrap(),
            json
        );
    }

    #[test]
    fn test_deserialize_third_party_network() {
        let json = json!({ "third_party_instance_id": "freenode" });
        assert_eq!(
            from_json_value::<RoomNetwork>(json).unwrap(),
            RoomNetwork::ThirdParty("freenode".to_string())
        );
    }

    #[test]
    fn test_deserialize_include_all_networks_and_third_party_exclusivity() {
        let json = json!({ "include_all_networks": true, "third_party_instance_id": "freenode" });
        assert_eq!(
            from_json_value::<RoomNetwork>(json)
                .unwrap_err()
                .to_string()
                .as_str(),
            "`include_all_networks = true` and `third_party_instance_id` are mutually exclusive."
        );
    }
}
