//! Endpoints for managing message retention periods

use std::fmt::{Display, Formatter, Result as FmtResult};

use ruma_common::OwnedRoomId;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, Unexpected},
};

pub mod get_retention_configuration;

/// Represents one or all rooms of a homeserver.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[allow(clippy::exhaustive_enums)]
pub enum RoomIdOrAllRooms {
    /// Represents a specific room ID.
    RoomId(OwnedRoomId),

    /// Represents all rooms on a homeserver.
    AllRooms,
}

impl Display for RoomIdOrAllRooms {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            RoomIdOrAllRooms::RoomId(room_id) => write!(f, "{room_id}"),
            RoomIdOrAllRooms::AllRooms => write!(f, "*"),
        }
    }
}

impl From<OwnedRoomId> for RoomIdOrAllRooms {
    fn from(r: OwnedRoomId) -> Self {
        RoomIdOrAllRooms::RoomId(r)
    }
}

impl TryFrom<&str> for RoomIdOrAllRooms {
    type Error = &'static str;

    fn try_from(room_id_or_wildcard: &str) -> Result<Self, Self::Error> {
        if room_id_or_wildcard.is_empty() {
            Err("The Room identifier cannot be empty")
        } else if "*" == room_id_or_wildcard {
            Ok(RoomIdOrAllRooms::AllRooms)
        } else {
            Ok(RoomIdOrAllRooms::RoomId(
                room_id_or_wildcard
                    .try_into()
                    .map_err(|_| "The Room identifier needs to be a valid room id or *")?,
            ))
        }
    }
}

impl Serialize for RoomIdOrAllRooms {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::RoomId(room_id) => room_id.serialize(serializer),
            Self::AllRooms => serializer.serialize_str("*"),
        }
    }
}

impl<'de> Deserialize<'de> for RoomIdOrAllRooms {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = ruma_common::serde::deserialize_cow_str(deserializer)?;
        RoomIdOrAllRooms::try_from(s.as_ref())
            .map_err(|_| de::Error::invalid_value(Unexpected::Str(&s), &"a valid room ID or '*'"))
    }
}
