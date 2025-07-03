//! Common types for rooms.

use std::borrow::Cow;

use js_int::UInt;
use serde::{de, Deserialize, Serialize};
use serde_json::value::RawValue as RawJsonValue;

use crate::{
    directory::PublicRoomJoinRule,
    serde::{from_raw_json_value, StringEnum},
    space::SpaceRoomJoinRule,
    EventEncryptionAlgorithm, OwnedMxcUri, OwnedRoomAliasId, OwnedRoomId, PrivOwnedStr,
    RoomVersionId,
};

/// An enum of possible room types.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, Eq, StringEnum)]
#[non_exhaustive]
pub enum RoomType {
    /// Defines the room as a space.
    #[ruma_enum(rename = "m.space")]
    Space,

    /// Defines the room as a custom type.
    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The summary of a room's state.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RoomSummary {
    /// The ID of the room.
    pub room_id: OwnedRoomId,

    /// The canonical alias of the room, if any.
    ///
    /// If the `compat-empty-string-null` cargo feature is enabled, this field being an empty
    /// string in JSON will result in `None` here during deserialization.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "compat-empty-string-null",
        serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
    )]
    pub canonical_alias: Option<OwnedRoomAliasId>,

    /// The name of the room, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The topic of the room, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,

    /// The URL for the room's avatar, if one is set.
    ///
    /// If you activate the `compat-empty-string-null` feature, this field being an empty string in
    /// JSON will result in `None` here during deserialization.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "compat-empty-string-null",
        serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
    )]
    pub avatar_url: Option<OwnedMxcUri>,

    /// The type of room from `m.room.create`, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_type: Option<RoomType>,

    /// The number of members joined to the room.
    pub num_joined_members: UInt,

    /// The join rule of the room.
    #[serde(flatten, skip_serializing_if = "ruma_common::serde::is_default")]
    pub join_rule: JoinRuleSummary,

    /// Whether the room may be viewed by users without joining.
    pub world_readable: bool,

    /// Whether guest users may join the room and participate in it.
    ///
    /// If they can, they will be subject to ordinary power level rules like any other user.
    pub guest_can_join: bool,

    /// If the room is encrypted, the algorithm used for this room.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption: Option<EventEncryptionAlgorithm>,

    /// The version of the room.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_version: Option<RoomVersionId>,
}

impl RoomSummary {
    /// Construct a new `RoomSummary` with the given required fields.
    pub fn new(
        room_id: OwnedRoomId,
        join_rule: JoinRuleSummary,
        guest_can_join: bool,
        num_joined_members: UInt,
        world_readable: bool,
    ) -> Self {
        Self {
            room_id,
            canonical_alias: None,
            name: None,
            topic: None,
            avatar_url: None,
            room_type: None,
            num_joined_members,
            join_rule,
            world_readable,
            guest_can_join,
            encryption: None,
            room_version: None,
        }
    }
}

impl<'de> Deserialize<'de> for RoomSummary {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        /// Helper type to deserialize [`RoomSummary`] because using `flatten` on `join_rule`
        /// returns an error.
        #[derive(Deserialize)]
        struct RoomSummaryDeHelper {
            room_id: OwnedRoomId,
            #[cfg_attr(
                feature = "compat-empty-string-null",
                serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
            )]
            canonical_alias: Option<OwnedRoomAliasId>,
            name: Option<String>,
            topic: Option<String>,
            #[cfg_attr(
                feature = "compat-empty-string-null",
                serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
            )]
            avatar_url: Option<OwnedMxcUri>,
            room_type: Option<RoomType>,
            num_joined_members: UInt,
            world_readable: bool,
            guest_can_join: bool,
            encryption: Option<EventEncryptionAlgorithm>,
            room_version: Option<RoomVersionId>,
        }

        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let RoomSummaryDeHelper {
            room_id,
            canonical_alias,
            name,
            topic,
            avatar_url,
            room_type,
            num_joined_members,
            world_readable,
            guest_can_join,
            encryption,
            room_version,
        } = from_raw_json_value(&json)?;
        let join_rule: JoinRuleSummary = from_raw_json_value(&json)?;

        Ok(Self {
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
            encryption,
            room_version,
        })
    }
}

/// The rule used for users wishing to join a room.
///
/// In contrast to the regular `JoinRule` in `ruma_events`, this enum does holds only simplified
/// conditions for joining restricted rooms.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[serde(tag = "join_rule", rename_all = "snake_case")]
pub enum JoinRuleSummary {
    /// A user who wishes to join the room must first receive an invite to the room from someone
    /// already inside of the room.
    Invite,

    /// Users can join the room if they are invited, or they can request an invite to the room.
    ///
    /// They can be allowed (invited) or denied (kicked/banned) access.
    Knock,

    /// Reserved but not yet implemented by the Matrix specification.
    Private,

    /// Users can join the room if they are invited, or if they meet one of the conditions
    /// described in the [`RestrictedSummary`].
    Restricted(RestrictedSummary),

    /// Users can join the room if they are invited, or if they meet one of the conditions
    /// described in the [`RestrictedSummary`], or they can request an invite to the room.
    KnockRestricted(RestrictedSummary),

    /// Anyone can join the room without any prior action.
    #[default]
    Public,

    #[doc(hidden)]
    #[serde(skip_serializing)]
    _Custom(PrivOwnedStr),
}

impl JoinRuleSummary {
    /// Returns the string name of this `JoinRule`.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Invite => "invite",
            Self::Knock => "knock",
            Self::Private => "private",
            Self::Restricted(_) => "restricted",
            Self::KnockRestricted(_) => "knock_restricted",
            Self::Public => "public",
            Self::_Custom(rule) => &rule.0,
        }
    }
}

impl From<JoinRuleSummary> for PublicRoomJoinRule {
    fn from(value: JoinRuleSummary) -> Self {
        match value {
            JoinRuleSummary::Invite => Self::Invite,
            JoinRuleSummary::Knock => Self::Knock,
            JoinRuleSummary::Private => Self::Private,
            JoinRuleSummary::Restricted(_) => Self::Restricted,
            JoinRuleSummary::KnockRestricted(_) => Self::KnockRestricted,
            JoinRuleSummary::Public => Self::Public,
            JoinRuleSummary::_Custom(custom) => Self::_Custom(custom),
        }
    }
}

impl From<JoinRuleSummary> for SpaceRoomJoinRule {
    fn from(value: JoinRuleSummary) -> Self {
        match value {
            JoinRuleSummary::Invite => Self::Invite,
            JoinRuleSummary::Knock => Self::Knock,
            JoinRuleSummary::Private => Self::Private,
            JoinRuleSummary::Restricted(_) => Self::Restricted,
            JoinRuleSummary::KnockRestricted(_) => Self::KnockRestricted,
            JoinRuleSummary::Public => Self::Public,
            JoinRuleSummary::_Custom(custom) => Self::_Custom(custom),
        }
    }
}

impl<'de> Deserialize<'de> for JoinRuleSummary {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json: Box<RawJsonValue> = Box::deserialize(deserializer)?;

        #[derive(Deserialize)]
        struct ExtractType<'a> {
            #[serde(borrow)]
            join_rule: Option<Cow<'a, str>>,
        }

        let Some(join_rule) = serde_json::from_str::<ExtractType<'_>>(json.get())
            .map_err(de::Error::custom)?
            .join_rule
        else {
            return Ok(Self::default());
        };

        match join_rule.as_ref() {
            "invite" => Ok(Self::Invite),
            "knock" => Ok(Self::Knock),
            "private" => Ok(Self::Private),
            "restricted" => from_raw_json_value(&json).map(Self::Restricted),
            "knock_restricted" => from_raw_json_value(&json).map(Self::KnockRestricted),
            "public" => Ok(Self::Public),
            _ => Ok(Self::_Custom(PrivOwnedStr(join_rule.into()))),
        }
    }
}

/// A summary of the conditions for joining a restricted room.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RestrictedSummary {
    /// The room IDs which are specified by the join rules.
    #[serde(default)]
    pub allowed_room_ids: Vec<OwnedRoomId>,
}

impl RestrictedSummary {
    /// Constructs a new `RestrictedSummary` with the given room IDs.
    pub fn new(allowed_room_ids: Vec<OwnedRoomId>) -> Self {
        Self { allowed_room_ids }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use js_int::uint;
    use ruma_common::owned_room_id;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{JoinRuleSummary, RestrictedSummary, RoomSummary};

    #[test]
    fn deserialize_summary_no_join_rule() {
        let json = json!({
            "room_id": "!room:localhost",
            "num_joined_members": 5,
            "world_readable": false,
            "guest_can_join": false,
        });

        let summary: RoomSummary = from_json_value(json).unwrap();
        assert_eq!(summary.room_id, "!room:localhost");
        assert_eq!(summary.num_joined_members, uint!(5));
        assert!(!summary.world_readable);
        assert!(!summary.guest_can_join);
        assert_matches!(summary.join_rule, JoinRuleSummary::Public);
    }

    #[test]
    fn deserialize_summary_private_join_rule() {
        let json = json!({
            "room_id": "!room:localhost",
            "num_joined_members": 5,
            "world_readable": false,
            "guest_can_join": false,
            "join_rule": "private",
        });

        let summary: RoomSummary = from_json_value(json).unwrap();
        assert_eq!(summary.room_id, "!room:localhost");
        assert_eq!(summary.num_joined_members, uint!(5));
        assert!(!summary.world_readable);
        assert!(!summary.guest_can_join);
        assert_matches!(summary.join_rule, JoinRuleSummary::Private);
    }

    #[test]
    fn deserialize_summary_restricted_join_rule() {
        let json = json!({
            "room_id": "!room:localhost",
            "num_joined_members": 5,
            "world_readable": false,
            "guest_can_join": false,
            "join_rule": "restricted",
            "allowed_room_ids": ["!otherroom:localhost"],
        });

        let summary: RoomSummary = from_json_value(json).unwrap();
        assert_eq!(summary.room_id, "!room:localhost");
        assert_eq!(summary.num_joined_members, uint!(5));
        assert!(!summary.world_readable);
        assert!(!summary.guest_can_join);
        assert_matches!(summary.join_rule, JoinRuleSummary::Restricted(restricted));
        assert_eq!(restricted.allowed_room_ids.len(), 1);
    }

    #[test]
    fn deserialize_summary_restricted_join_rule_no_allowed_room_ids() {
        let json = json!({
            "room_id": "!room:localhost",
            "num_joined_members": 5,
            "world_readable": false,
            "guest_can_join": false,
            "join_rule": "restricted",
        });

        let summary: RoomSummary = from_json_value(json).unwrap();
        assert_eq!(summary.room_id, "!room:localhost");
        assert_eq!(summary.num_joined_members, uint!(5));
        assert!(!summary.world_readable);
        assert!(!summary.guest_can_join);
        assert_matches!(summary.join_rule, JoinRuleSummary::Restricted(restricted));
        assert_eq!(restricted.allowed_room_ids.len(), 0);
    }

    #[test]
    fn serialize_summary_knock_join_rule() {
        let summary = RoomSummary::new(
            owned_room_id!("!room:localhost"),
            JoinRuleSummary::Knock,
            false,
            uint!(5),
            false,
        );

        assert_eq!(
            to_json_value(&summary).unwrap(),
            json!({
                "room_id": "!room:localhost",
                "num_joined_members": 5,
                "world_readable": false,
                "guest_can_join": false,
                "join_rule": "knock",
            })
        );
    }

    #[test]
    fn serialize_summary_restricted_join_rule() {
        let summary = RoomSummary::new(
            owned_room_id!("!room:localhost"),
            JoinRuleSummary::Restricted(RestrictedSummary::new(vec![owned_room_id!(
                "!otherroom:localhost"
            )])),
            false,
            uint!(5),
            false,
        );

        assert_eq!(
            to_json_value(&summary).unwrap(),
            json!({
                "room_id": "!room:localhost",
                "num_joined_members": 5,
                "world_readable": false,
                "guest_can_join": false,
                "join_rule": "restricted",
                "allowed_room_ids": ["!otherroom:localhost"],
            })
        );
    }
}
