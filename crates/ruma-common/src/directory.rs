//! Common types for room directory endpoints.

use js_int::UInt;
use serde::{Deserialize, Serialize};

mod filter_room_type_serde;
mod room_network_serde;

use crate::{
    OwnedMxcUri, OwnedRoomAliasId, OwnedRoomId, PrivOwnedStr,
    room::{JoinRuleKind, RoomSummary, RoomType},
};

/// A chunk of a room list response, describing one room.
///
/// To create an instance of this type, first create a [`PublicRoomsChunkInit`] and convert it via
/// `PublicRoomsChunk::from` / `.into()`. It is also possible to construct this type from or convert
/// it to a [`RoomSummary`].
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct PublicRoomsChunk {
    /// The canonical alias of the room, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "compat-empty-string-null",
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
    /// If you activate the `compat-empty-string-null` feature, this field being an empty string in
    /// JSON will result in `None` here during deserialization.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "compat-empty-string-null",
        serde(default, deserialize_with = "crate::serde::empty_string_as_none")
    )]
    pub avatar_url: Option<OwnedMxcUri>,

    /// The join rule of the room.
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub join_rule: JoinRuleKind,

    /// The type of room from `m.room.create`, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_type: Option<RoomType>,
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
            join_rule: JoinRuleKind::default(),
            room_type: None,
        }
    }
}

impl From<RoomSummary> for PublicRoomsChunk {
    fn from(value: RoomSummary) -> Self {
        let RoomSummary {
            room_id,
            canonical_alias,
            name,
            topic,
            avatar_url,
            room_type,
            num_joined_members,
            join_rule,
            world_readable,
            guest_can_join,
            ..
        } = value;

        Self {
            canonical_alias,
            name,
            num_joined_members,
            room_id,
            topic,
            world_readable,
            guest_can_join,
            avatar_url,
            join_rule: join_rule.as_str().into(),
            room_type,
        }
    }
}

impl From<PublicRoomsChunk> for RoomSummary {
    fn from(value: PublicRoomsChunk) -> Self {
        let PublicRoomsChunk {
            room_id,
            canonical_alias,
            name,
            topic,
            avatar_url,
            room_type,
            num_joined_members,
            join_rule,
            world_readable,
            guest_can_join,
        } = value;

        Self {
            canonical_alias,
            name,
            num_joined_members,
            room_id,
            topic,
            world_readable,
            guest_can_join,
            avatar_url,
            join_rule: join_rule.into(),
            room_type,
            encryption: None,
            room_version: None,
        }
    }
}

/// A filter for public rooms lists.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct Filter {
    /// A string to search for in the room metadata, e.g. name, topic, canonical alias etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generic_search_term: Option<String>,

    /// The room types to include in the results.
    ///
    /// Includes all room types if it is empty.
    ///
    /// If the `compat-null` feature is enabled, a `null` value is allowed in deserialization, and
    /// treated the same way as an empty list.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "compat-null", serde(deserialize_with = "crate::serde::none_as_default"))]
    pub room_types: Vec<RoomTypeFilter>,
}

impl Filter {
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
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum RoomNetwork {
    /// Return rooms from the Matrix network.
    #[default]
    Matrix,

    /// Return rooms from all the networks/protocols the homeserver knows about.
    All,

    /// Return rooms from a specific third party network/protocol.
    ThirdParty(String),
}

/// An enum of possible room types to filter.
///
/// This type can hold an arbitrary string. To build this with a custom value, convert it from an
/// `Option<string>` with `::from()` / `.into()`. [`RoomTypeFilter::Default`] can be constructed
/// from `None`.
///
/// To check for values that are not available as a documented variant here, use its string
/// representation, obtained through [`.as_str()`](Self::as_str()).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RoomTypeFilter {
    /// The default room type, defined without a `room_type`.
    Default,

    /// A space.
    Space,

    /// A custom room type.
    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl RoomTypeFilter {
    /// Get the string representation of this `RoomTypeFilter`.
    ///
    /// [`RoomTypeFilter::Default`] returns `None`.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            RoomTypeFilter::Default => None,
            RoomTypeFilter::Space => Some("m.space"),
            RoomTypeFilter::_Custom(s) => Some(&s.0),
        }
    }
}

impl<T> From<Option<T>> for RoomTypeFilter
where
    T: AsRef<str> + Into<Box<str>>,
{
    fn from(s: Option<T>) -> Self {
        match s {
            None => Self::Default,
            Some(s) => match s.as_ref() {
                "m.space" => Self::Space,
                _ => Self::_Custom(PrivOwnedStr(s.into())),
            },
        }
    }
}

impl From<Option<RoomType>> for RoomTypeFilter {
    fn from(t: Option<RoomType>) -> Self {
        match t {
            None => Self::Default,
            Some(s) => match s {
                RoomType::Space => Self::Space,
                _ => Self::from(Some(s.as_str())),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use serde_json::{from_value as from_json_value, json};

    use super::{Filter, RoomNetwork, RoomTypeFilter};
    use crate::{assert_to_canonical_json_eq, room::RoomType};

    #[test]
    fn test_from_room_type() {
        let test = RoomType::Space;
        let other: RoomTypeFilter = RoomTypeFilter::from(Some(test));
        assert_eq!(other, RoomTypeFilter::Space);
    }

    #[test]
    fn serialize_matrix_network_only() {
        assert_to_canonical_json_eq!(RoomNetwork::Matrix, json!({}));
    }

    #[test]
    fn deserialize_matrix_network_only() {
        let json = json!({ "include_all_networks": false });
        assert_eq!(from_json_value::<RoomNetwork>(json).unwrap(), RoomNetwork::Matrix);
    }

    #[test]
    fn serialize_default_network_is_empty() {
        assert_to_canonical_json_eq!(RoomNetwork::default(), json!({}));
    }

    #[test]
    fn deserialize_empty_network_is_default() {
        let json = json!({});
        assert_eq!(from_json_value::<RoomNetwork>(json).unwrap(), RoomNetwork::Matrix);
    }

    #[test]
    fn serialize_include_all_networks() {
        assert_to_canonical_json_eq!(RoomNetwork::All, json!({ "include_all_networks": true }));
    }

    #[test]
    fn deserialize_include_all_networks() {
        let json = json!({ "include_all_networks": true });
        assert_eq!(from_json_value::<RoomNetwork>(json).unwrap(), RoomNetwork::All);
    }

    #[test]
    fn serialize_third_party_network() {
        assert_to_canonical_json_eq!(
            RoomNetwork::ThirdParty("freenode".to_owned()),
            json!({ "third_party_instance_id": "freenode" }),
        );
    }

    #[test]
    fn deserialize_third_party_network() {
        let json = json!({ "third_party_instance_id": "freenode" });
        assert_eq!(
            from_json_value::<RoomNetwork>(json).unwrap(),
            RoomNetwork::ThirdParty("freenode".into())
        );
    }

    #[test]
    fn deserialize_include_all_networks_and_third_party_exclusivity() {
        let json = json!({ "include_all_networks": true, "third_party_instance_id": "freenode" });
        assert_eq!(
            from_json_value::<RoomNetwork>(json).unwrap_err().to_string().as_str(),
            "`include_all_networks = true` and `third_party_instance_id` are mutually exclusive."
        );
    }

    #[test]
    fn serialize_filter_empty() {
        assert_to_canonical_json_eq!(Filter::default(), json!({}));
    }

    #[test]
    fn deserialize_filter_empty() {
        let json = json!({});
        let filter = from_json_value::<Filter>(json).unwrap();
        assert_eq!(filter.generic_search_term, None);
        assert_eq!(filter.room_types.len(), 0);
    }

    #[test]
    fn serialize_filter_room_types() {
        let filter = Filter {
            generic_search_term: None,
            room_types: vec![
                RoomTypeFilter::Default,
                RoomTypeFilter::Space,
                Some("custom_type").into(),
            ],
        };
        assert_to_canonical_json_eq!(
            filter,
            json!({ "room_types": [null, "m.space", "custom_type"] }),
        );
    }

    #[test]
    fn deserialize_filter_room_types() {
        let json = json!({ "room_types": [null, "m.space", "custom_type"] });
        let filter = from_json_value::<Filter>(json).unwrap();
        assert_eq!(filter.room_types.len(), 3);
        assert_eq!(filter.room_types[0], RoomTypeFilter::Default);
        assert_eq!(filter.room_types[1], RoomTypeFilter::Space);
        assert_matches!(&filter.room_types[2], RoomTypeFilter::_Custom(_));
        assert_eq!(filter.room_types[2].as_str(), Some("custom_type"));
    }
}
