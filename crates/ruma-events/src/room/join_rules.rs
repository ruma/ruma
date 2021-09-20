//! Types for the *m.room.join_rules* event.

#[cfg(feature = "unstable-pre-spec")]
use std::collections::BTreeMap;

use ruma_events_macros::EventContent;
#[cfg(feature = "unstable-pre-spec")]
use ruma_identifiers::RoomId;
use ruma_serde::StringEnum;
#[cfg(feature = "unstable-pre-spec")]
use serde::de::{DeserializeOwned, Deserializer, Error};
use serde::{Deserialize, Serialize};
#[cfg(feature = "unstable-pre-spec")]
use serde_json::{value::RawValue as RawJsonValue, Value as JsonValue};

use crate::StateEvent;

/// Describes how users are allowed to join the room.
pub type JoinRulesEvent = StateEvent<JoinRulesEventContent>;

/// The payload for `JoinRulesEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.join_rules", kind = State)]
pub struct JoinRulesEventContent {
    /// The type of rules used for users wishing to join this room.
    #[ruma_event(skip_redaction)]
    pub join_rule: JoinRule,

    /// Allow rules used for the `restricted` join rule.
    #[cfg(feature = "unstable-pre-spec")]
    #[serde(default)]
    #[ruma_event(skip_redaction)]
    pub allow: Vec<AllowRule>,
}

impl JoinRulesEventContent {
    /// Creates a new `JoinRulesEventContent` with the given rule.
    pub fn new(join_rule: JoinRule) -> Self {
        Self {
            join_rule,
            #[cfg(feature = "unstable-pre-spec")]
            allow: Vec::new(),
        }
    }

    /// Creates a new `JoinRulesEventContent` with the restricted rule and the given set of allow
    /// rules.
    #[cfg(feature = "unstable-pre-spec")]
    pub fn restricted(allow: Vec<AllowRule>) -> Self {
        Self { join_rule: JoinRule::Restricted, allow }
    }
}

/// The rule used for users wishing to join this room.
///
/// This type can hold an arbitrary string. To check for formats that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "lowercase")]
#[non_exhaustive]
pub enum JoinRule {
    /// A user who wishes to join the room must first receive an invite to the room from someone
    /// already inside of the room.
    Invite,

    /// Reserved but not yet implemented by the Matrix specification.
    Knock,

    /// Reserved but not yet implemented by the Matrix specification.
    Private,

    /// Users can join the room if they are invited, or if they meet any of the conditions
    /// described in a set of [`AllowRule`]s.
    #[cfg(feature = "unstable-pre-spec")]
    Restricted,

    /// Anyone can join the room without any prior action.
    Public,

    #[doc(hidden)]
    _Custom(String),
}

impl JoinRule {
    /// Creates a string slice from this `JoinRule`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

/// An allow rule which defines a condition that allows joining a room.
#[cfg(feature = "unstable-pre-spec")]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type")]
pub enum AllowRule {
    /// Joining is allowed if a user is already a member of the romm with the id `room_id`.
    #[serde(rename = "m.room_membership")]
    RoomMembership(RoomMembership),

    #[doc(hidden)]
    _Custom(CustomAllowRule),
}

/// Allow rule which grants permission to join based on the membership of another room.
#[cfg(feature = "unstable-pre-spec")]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RoomMembership {
    /// The id of the room which being a member of grants permission to join another room.
    pub room_id: RoomId,
}

#[cfg(feature = "unstable-pre-spec")]
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct CustomAllowRule {
    #[serde(rename = "type")]
    rule_type: String,
    #[serde(flatten)]
    extra: BTreeMap<String, JsonValue>,
}

#[cfg(feature = "unstable-pre-spec")]
impl<'de> Deserialize<'de> for AllowRule {
    fn deserialize<D>(deserializer: D) -> Result<AllowRule, D::Error>
    where
        D: Deserializer<'de>,
    {
        fn from_raw_json_value<T: DeserializeOwned, E: Error>(raw: &RawJsonValue) -> Result<T, E> {
            serde_json::from_str(raw.get()).map_err(E::custom)
        }

        let json: Box<RawJsonValue> = Box::deserialize(deserializer)?;

        // Extracts the `type` value.
        #[derive(Deserialize)]
        struct ExtractType {
            rule_type: Option<String>,
        }

        // Get the value of `type` if present.
        let rule_type = serde_json::from_str::<ExtractType>(json.get())
            .map_err(serde::de::Error::custom)?
            .rule_type;

        match rule_type.as_deref() {
            Some("m.room_membership") => from_raw_json_value(&json).map(Self::RoomMembership),
            Some(_) => from_raw_json_value(&json).map(Self::_Custom),
            None => Err(D::Error::missing_field("type")),
        }
    }
}
