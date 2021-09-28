//! Types for the *m.room.join_rules* event.

use ruma_events_macros::EventContent;
#[cfg(feature = "unstable-pre-spec")]
use ruma_identifiers::RoomId;
#[cfg(feature = "unstable-pre-spec")]
use serde::de::DeserializeOwned;
use serde::{
    de::{Deserializer, Error},
    Deserialize, Serialize,
};
use serde_json::value::RawValue as RawJsonValue;
#[cfg(feature = "unstable-pre-spec")]
use serde_json::Value as JsonValue;
use std::borrow::Cow;
#[cfg(feature = "unstable-pre-spec")]
use std::collections::BTreeMap;

use crate::StateEvent;

/// Describes how users are allowed to join the room.
pub type JoinRulesEvent = StateEvent<JoinRulesEventContent>;

/// The payload for `JoinRulesEvent`.
#[derive(Clone, Debug, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.join_rules", kind = State)]
pub struct JoinRulesEventContent {
    /// The type of rules used for users wishing to join this room.
    #[ruma_event(skip_redaction)]
    #[serde(flatten)]
    pub join_rule: JoinRule,
}

impl JoinRulesEventContent {
    /// Creates a new `JoinRulesEventContent` with the given rule.
    pub fn new(join_rule: JoinRule) -> Self {
        Self { join_rule }
    }

    /// Creates a new `JoinRulesEventContent` with the restricted rule and the given set of allow
    /// rules.
    #[cfg(feature = "unstable-pre-spec")]
    pub fn restricted(allow: Vec<AllowRule>) -> Self {
        Self { join_rule: JoinRule::Restricted(Restricted::new(allow)) }
    }
}

impl<'de> Deserialize<'de> for JoinRulesEventContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let join_rule = JoinRule::deserialize(deserializer)?;
        Ok(JoinRulesEventContent { join_rule })
    }
}

/// The rule used for users wishing to join this room.
///
/// This type can hold an arbitrary string. To check for formats that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "join_rule")]
pub enum JoinRule {
    /// A user who wishes to join the room must first receive an invite to the room from someone
    /// already inside of the room.
    #[serde(rename = "invite")]
    Invite,

    /// Reserved but not yet implemented by the Matrix specification.
    #[serde(rename = "knock")]
    Knock,

    /// Reserved but not yet implemented by the Matrix specification.
    #[serde(rename = "private")]
    Private,

    /// Users can join the room if they are invited, or if they meet any of the conditions
    /// described in a set of [`AllowRule`]s.
    #[cfg(feature = "unstable-pre-spec")]
    #[serde(rename = "restricted")]
    Restricted(Restricted),

    /// Anyone can join the room without any prior action.
    #[serde(rename = "public")]
    Public,

    #[doc(hidden)]
    #[serde(skip_serializing)]
    _Custom(String),
}

impl JoinRule {
    /// Returns the string name of this `JoinRule`
    pub fn as_str(&self) -> &str {
        match self {
            JoinRule::Invite => "invite",
            JoinRule::Knock => "knock",
            JoinRule::Private => "private",
            #[cfg(feature = "unstable-pre-spec")]
            JoinRule::Restricted(_) => "restricted",
            JoinRule::Public => "public",
            JoinRule::_Custom(rule) => rule,
        }
    }
}

impl<'de> Deserialize<'de> for JoinRule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[cfg(feature = "unstable-pre-spec")]
        fn from_raw_json_value<T: DeserializeOwned, E: Error>(raw: &RawJsonValue) -> Result<T, E> {
            serde_json::from_str(raw.get()).map_err(E::custom)
        }

        let json: Box<RawJsonValue> = Box::deserialize(deserializer)?;

        #[derive(Deserialize)]
        struct ExtractType<'a> {
            join_rule: Option<Cow<'a, str>>,
        }

        let join_rule = serde_json::from_str::<ExtractType<'_>>(json.get())
            .map_err(serde::de::Error::custom)?
            .join_rule
            .ok_or_else(|| D::Error::missing_field("join_rule"))?;

        match join_rule.as_ref() {
            "invite" => Ok(Self::Invite),
            "knock" => Ok(Self::Knock),
            "private" => Ok(Self::Private),
            #[cfg(feature = "unstable-pre-spec")]
            "restricted" => from_raw_json_value(&json).map(Self::Restricted),
            "public" => Ok(Self::Public),
            _ => Ok(Self::_Custom(join_rule.into_owned())),
        }
    }
}

/// Configuration of the `Restricted` join rule.
#[cfg(feature = "unstable-pre-spec")]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Restricted {
    /// Allow rules which describe conditions that allow joining a room.
    allow: Vec<AllowRule>,
}

#[cfg(feature = "unstable-pre-spec")]
impl Restricted {
    /// Constructs a new rule set for restricted rooms with the given rules.
    pub fn new(allow: Vec<AllowRule>) -> Self {
        Self { allow }
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

#[cfg(feature = "unstable-pre-spec")]
impl AllowRule {
    /// Constructs an `AllowRule` with membership of the room with the given id as its predicate.
    pub fn room_membership(room_id: RoomId) -> Self {
        Self::RoomMembership(RoomMembership::new(room_id))
    }
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
impl RoomMembership {
    /// Constructs a new room membership rule for the given room id.
    pub fn new(room_id: RoomId) -> Self {
        Self { room_id }
    }
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
        struct ExtractType<'a> {
            #[serde(borrow, rename = "type")]
            rule_type: Option<Cow<'a, str>>,
        }

        // Get the value of `type` if present.
        let rule_type = serde_json::from_str::<ExtractType<'_>>(json.get())
            .map_err(serde::de::Error::custom)?
            .rule_type;

        match rule_type.as_deref() {
            Some("m.room_membership") => from_raw_json_value(&json).map(Self::RoomMembership),
            Some(_) => from_raw_json_value(&json).map(Self::_Custom),
            None => Err(D::Error::missing_field("type")),
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "unstable-pre-spec")]
    use super::AllowRule;
    use super::{JoinRule, JoinRulesEventContent};
    #[cfg(feature = "unstable-pre-spec")]
    use ruma_identifiers::room_id;

    #[test]
    fn deserialize() {
        let json = r#"{"join_rule": "public"}"#;
        let event: JoinRulesEventContent = serde_json::from_str(json).unwrap();
        assert!(matches!(event, JoinRulesEventContent { join_rule: JoinRule::Public }));
    }

    #[cfg(feature = "unstable-pre-spec")]
    #[test]
    fn deserialize_unstable() {
        let json = r#"{
            "join_rule": "restricted",
            "allow": [
                {
                    "type": "m.room_membership",
                    "room_id": "!mods:example.org"
                },
                {
                    "type": "m.room_membership",
                    "room_id": "!users:example.org"
                }
            ]
        }"#;
        let event: JoinRulesEventContent = serde_json::from_str(json).unwrap();
        match event.join_rule {
            JoinRule::Restricted(restricted) => assert_eq!(
                restricted.allow,
                &[
                    AllowRule::room_membership(room_id!("!mods:example.org")),
                    AllowRule::room_membership(room_id!("!users:example.org"))
                ]
            ),
            rule => panic!("Deserialized to wrong variant: {:?}", rule),
        }
    }
}
