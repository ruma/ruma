//! Types for the *m.presence* event.

use std::fmt::{Display, Formatter, Error as FmtError};
use std::str::FromStr;

use ruma_identifiers::{EventId, UserId};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use {Event, ParseError, Visitor};

/// Informs the client of a user's presence state change.
pub type PresenceEvent = Event<PresenceEventContent, PresenceEventExtraContent>;

/// The payload of a `PresenceEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct PresenceEventContent {
    /// The current avatar URL for this user.
    pub avatar_url: Option<String>,

    /// Whether or not the user is currently active.
    pub currently_active: bool,

    /// The current display name for this user.
    pub displayname: Option<String>,

    /// The last time since this used performed some action, in milliseconds.
    pub last_active_ago: Option<u64>,

    /// The presence state for this user.
    pub presence: PresenceState,

    /// The unique identifier for the user associated with this event.
    pub user_id: UserId,
}

/// A description of a user's connectivity and availability for chat.
#[derive(Debug, PartialEq)]
pub enum PresenceState {
    /// Disconnected from the service.
    Offline,

    /// Connected to the service.
    Online,

    /// Connected to the service but not available for chat.
    Unavailable,
}

/// Extra content for a `PresenceEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct PresenceEventExtraContent {
    /// The unique identifier for the event.
    pub event_id: EventId,
}

impl Display for PresenceState {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        let presence_state_str = match *self {
            PresenceState::Offline => "offline",
            PresenceState::Online => "online",
            PresenceState::Unavailable => "unavailable",
        };

        write!(f, "{}", presence_state_str)
    }
}

impl FromStr for PresenceState {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "offline" => Ok(PresenceState::Offline),
            "online" => Ok(PresenceState::Online),
            "unavailable" => Ok(PresenceState::Unavailable),
            _ => Err(ParseError),
        }
    }
}

impl Serialize for PresenceState {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl Deserialize for PresenceState {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
        deserializer.deserialize_str(Visitor::new())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_str, to_string};

    use super::PresenceState;

    #[test]
    fn presence_states_serialize_to_display_form() {
        assert_eq!(
            to_string(&PresenceState::Offline).unwrap(),
            r#""offline""#
        );
    }

    #[test]
    fn presence_states_deserialize_from_display_form() {
        assert_eq!(
            from_str::<PresenceState>(r#""offline""#).unwrap(),
            PresenceState::Offline
        );
    }

    #[test]
    fn invalid_presence_states_fail_deserialization() {
        assert!(from_str::<PresenceState>(r#""bad""#).is_err());
    }
}
