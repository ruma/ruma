//! Types for the *m.room.join_rules* event.

use std::fmt::{Display, Formatter, Error as FmtError};
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Error as SerdeError, Serialize, Serializer};
use serde::de::Visitor;

use StateEvent;

/// Describes how users are allowed to join the room.
pub type JoinRulesEvent = StateEvent<JoinRulesEventContent, ()>;

/// The payload of a `JoinRulesEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct JoinRulesEventContent {
    /// The type of rules used for users wishing to join this room.
    pub join_rule: JoinRule,
}

/// The rule used for users wishing to join this room.
#[derive(Debug, PartialEq)]
pub enum JoinRule {
    /// A user who wishes to join the room must first receive an invite to the room from someone
    /// already inside of the room.
    Invite,

    /// Reserved but not yet implemented by the Matrix specification.
    Knock,

    /// Reserved but not yet implemented by the Matrix specification.
    Private,

    /// Anyone can join the room without any prior action.
    Public,
}

/// An error when attempting to parse an invalid `JoinRule` from a string.
pub struct JoinRuleParseError;

impl Display for JoinRule {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        let join_rule_str = match *self {
            JoinRule::Invite => "invite",
            JoinRule::Knock => "knock",
            JoinRule::Private => "private",
            JoinRule::Public => "public",
        };

        write!(f, "{}", join_rule_str)
    }
}

impl FromStr for JoinRule {
    type Err = JoinRuleParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "invite" => Ok(JoinRule::Invite),
            "knock" => Ok(JoinRule::Knock),
            "private" => Ok(JoinRule::Private),
            "public" => Ok(JoinRule::Public),
            _ => Err(JoinRuleParseError),
        }
    }
}

impl Serialize for JoinRule {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl Deserialize for JoinRule {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
        struct JoinRuleVisitor;

        impl Visitor for JoinRuleVisitor {
            type Value = JoinRule;

            fn visit_str<E>(&mut self, v: &str) -> Result<Self::Value, E> where E: SerdeError {
                v.parse().map_err(|_| {
                    E::invalid_value(v)
                })
            }
        }

        deserializer.deserialize_str(JoinRuleVisitor)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_str, to_string};

    use super::JoinRule;

    #[test]
    fn join_rules_serialize_to_display_form() {
        assert_eq!(
            to_string(&JoinRule::Invite).unwrap(),
            r#""invite""#
        );
    }

    #[test]
    fn join_rules_deserialize_from_display_form() {
        assert_eq!(
            from_str::<JoinRule>(r#""invite""#).unwrap(),
            JoinRule::Invite
        );
    }

    #[test]
    fn invalid_join_rules_fail_deserialization() {
        assert!(from_str::<JoinRule>(r#""bad""#).is_err());
    }
}
