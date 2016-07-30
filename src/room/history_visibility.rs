//! Types for the *m.room.history_visibility* event.

use std::fmt::{Display, Formatter, Error as FmtError};
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Error as SerdeError, Serialize, Serializer};
use serde::de::Visitor;

use StateEvent;

/// This event controls whether a member of a room can see the events that happened in a room from
/// before they joined.
pub type HistoryVisibilityEvent = StateEvent<HistoryVisibilityEventContent, ()>;

/// The payload of a `HistoryVisibilityEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct HistoryVisibilityEventContent {
    /// Who can see the room history.
    pub history_visibility: HistoryVisibility,
}

/// Who can see a room's history.
#[derive(Debug, PartialEq)]
pub enum HistoryVisibility {
    /// Previous events are accessible to newly joined members from the point they were invited
    /// onwards. Events stop being accessible when the member's state changes to something other
    /// than *invite* or *join*.
    Invited,

    /// Previous events are accessible to newly joined members from the point they joined the room
    /// onwards. Events stop being accessible when the member's state changes to something other
    /// than *join*.
    Joined,

    /// Previous events are always accessible to newly joined members. All events in the room are
    /// accessible, even those sent when the member was not a part of the room.
    Shared,

    /// All events while this is the `HistoryVisibility` value may be shared by any
    /// participating homeserver with anyone, regardless of whether they have ever joined the room.
    WorldReadable,
}

/// An error when attempting to parse an invalid `HistoryVisibility` from a string.
pub struct HistoryVisibilityParseError;

impl Display for HistoryVisibility {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        let history_visibility_str = match *self {
            HistoryVisibility::Invited => "invited",
            HistoryVisibility::Joined => "joined",
            HistoryVisibility::Shared => "shared",
            HistoryVisibility::WorldReadable => "world_readable",
        };

        write!(f, "{}", history_visibility_str)
    }
}

impl FromStr for HistoryVisibility {
    type Err = HistoryVisibilityParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "invited" => Ok(HistoryVisibility::Invited),
            "joined" => Ok(HistoryVisibility::Joined),
            "shared" => Ok(HistoryVisibility::Shared),
            "world_readable" => Ok(HistoryVisibility::WorldReadable),
            _ => Err(HistoryVisibilityParseError),
        }
    }
}

impl Serialize for HistoryVisibility {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl Deserialize for HistoryVisibility {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
        struct HistoryVisibilityVisitor;

        impl Visitor for HistoryVisibilityVisitor {
            type Value = HistoryVisibility;

            fn visit_str<E>(&mut self, v: &str) -> Result<Self::Value, E> where E: SerdeError {
                v.parse().map_err(|_| {
                    E::invalid_value(v)
                })
            }
        }

        deserializer.deserialize_str(HistoryVisibilityVisitor)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_str, to_string};

    use super::HistoryVisibility;

    #[test]
    fn history_visibility_serializes_to_display_form() {
        assert_eq!(
            to_string(&HistoryVisibility::Invited).unwrap(),
            r#""invited""#
        );
    }

    #[test]
    fn history_visibility_deserializes_from_display_form() {
        assert_eq!(
            from_str::<HistoryVisibility>(r#""invited""#).unwrap(),
            HistoryVisibility::Invited
        );
    }

    #[test]
    fn invalid_history_visibility_fails_deserialization() {
        assert!(from_str::<HistoryVisibility>(r#""bad""#).is_err());
    }
}
