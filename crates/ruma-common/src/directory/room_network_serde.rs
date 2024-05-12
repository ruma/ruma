use std::fmt;

use serde::{
    de::{Error, MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_json::Value as JsonValue;

use super::RoomNetwork;

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

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
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
                    include_all_networks = value.as_bool().unwrap_or(false);
                }
                "third_party_instance_id" => {
                    third_party_instance_id = value.as_str().map(|v| v.to_owned());
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
