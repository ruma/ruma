//! Types for the *m.room.guest_access* event.

use std::fmt::{Display, Formatter, Error as FmtError};
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Error as SerdeError, Serialize, Serializer};
use serde::de::Visitor;

use StateEvent;

/// Controls whether guest users are allowed to join rooms.
///
/// This event controls whether guest users are allowed to join rooms. If this event is absent,
/// servers should act as if it is present and has the value `GuestAccess::Forbidden`.
pub type GuestAccessEvent = StateEvent<GuestAccessEventContent, ()>;

/// The payload of a `GuestAccessEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct GuestAccessEventContent {
    /// A policy for guest user access to a room.
    pub guest_access: GuestAccess,
}

/// A policy for guest user access to a room.
#[derive(Debug, PartialEq)]
pub enum GuestAccess {
    /// Guests are allowed to join the room.
    CanJoin,

    /// Guests are not allowed to join the room.
    Forbidden,
}

/// An error when attempting to parse an invalid `PresenceState` from a string.
pub struct GuestAccessParseError;

impl Display for GuestAccess {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        let guest_access_str = match *self {
            GuestAccess::CanJoin => "can_join",
            GuestAccess::Forbidden => "forbidden",
        };

        write!(f, "{}", guest_access_str)
    }
}

impl FromStr for GuestAccess {
    type Err = GuestAccessParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "can_join" => Ok(GuestAccess::CanJoin),
            "forbidden" => Ok(GuestAccess::Forbidden),
            _ => Err(GuestAccessParseError),
        }
    }
}

impl Serialize for GuestAccess {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl Deserialize for GuestAccess {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
        struct GuestAccessVisitor;

        impl Visitor for GuestAccessVisitor {
            type Value = GuestAccess;

            fn visit_str<E>(&mut self, v: &str) -> Result<Self::Value, E> where E: SerdeError {
                v.parse().map_err(|_| {
                    E::invalid_value(v)
                })
            }
        }

        deserializer.deserialize_str(GuestAccessVisitor)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_str, to_string};

    use super::GuestAccess;

    #[test]
    fn guest_access_serializes_to_display_form() {
        assert_eq!(
            to_string(&GuestAccess::CanJoin).unwrap(),
            r#""can_join""#
        );
    }

    #[test]
    fn guest_access_deserializes_from_display_form() {
        assert_eq!(
            from_str::<GuestAccess>(r#""can_join""#).unwrap(),
            GuestAccess::CanJoin
        );
    }

    #[test]
    fn invalid_guest_access_fails_deserialization() {
        assert!(from_str::<GuestAccess>(r#""bad""#).is_err());
    }
}
