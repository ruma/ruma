//! Types for the *m.room.power_levels* event.

use std::{collections::HashMap, convert::TryFrom, str::FromStr};

use js_int::{Int, UInt};
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use serde_json::Value;

use crate::{
    Event, EventType, InnerInvalidEvent, InvalidEvent, InvalidInput, RoomEvent, StateEvent,
};

/// Defines the power levels (privileges) of users in the room.
#[derive(Clone, Debug, PartialEq)]
pub struct PowerLevelsEvent {
    /// The event's content.
    pub content: PowerLevelsEventContent,

    /// The unique identifier for the event.
    pub event_id: EventId,

    /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver when this
    /// event was sent.
    pub origin_server_ts: UInt,

    /// The previous content for this state key, if any.
    pub prev_content: Option<PowerLevelsEventContent>,

    /// The unique identifier for the room associated with this event.
    pub room_id: Option<RoomId>,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: Option<Value>,

    /// The unique identifier for the user who sent this event.
    pub sender: UserId,

    /// A key that determines which piece of room state the event represents.
    pub state_key: String,
}

/// The payload for `PowerLevelsEvent`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PowerLevelsEventContent {
    /// The level required to ban a user.
    #[serde(default = "default_power_level")]
    pub ban: Int,

    /// The level required to send specific event types.
    ///
    /// This is a mapping from event type to power level required.
    pub events: HashMap<EventType, Int>,

    /// The default level required to send message events.
    #[serde(default)]
    pub events_default: Int,

    /// The level required to invite a user.
    #[serde(default = "default_power_level")]
    pub invite: Int,

    /// The level required to kick a user.
    #[serde(default = "default_power_level")]
    pub kick: Int,

    /// The level required to redact an event.
    #[serde(default = "default_power_level")]
    pub redact: Int,

    /// The default level required to send state events.
    #[serde(default = "default_power_level")]
    pub state_default: Int,

    /// The power levels for specific users.
    ///
    /// This is a mapping from `user_id` to power level for that user.
    pub users: HashMap<UserId, Int>,

    /// The default power level for every user in the room.
    #[serde(default)]
    pub users_default: Int,

    /// The power level requirements for specific notification types.
    ///
    /// This is a mapping from `key` to power level for that notifications key.
    pub notifications: NotificationPowerLevels,
}

impl FromStr for PowerLevelsEvent {
    type Err = crate::InvalidEvent;

    /// Attempt to create `Self` from parsing a string of JSON data.
    fn from_str(json: &str) -> Result<Self, Self::Err> {
        let raw = match serde_json::from_str::<raw::PowerLevelsEvent>(json) {
            Ok(raw) => raw,
            Err(error) => match serde_json::from_str::<serde_json::Value>(json) {
                Ok(value) => {
                    return Err(InvalidEvent(InnerInvalidEvent::Validation {
                        json: value,
                        message: error.to_string(),
                    }));
                }
                Err(error) => {
                    return Err(InvalidEvent(InnerInvalidEvent::Deserialization { error }));
                }
            },
        };

        Ok(Self {
            content: PowerLevelsEventContent {
                ban: raw.content.ban,
                events: raw.content.events,
                events_default: raw.content.events_default,
                invite: raw.content.invite,
                kick: raw.content.kick,
                redact: raw.content.redact,
                state_default: raw.content.state_default,
                users: raw.content.users,
                users_default: raw.content.users_default,
                notifications: raw.content.notifications,
            },
            event_id: raw.event_id,
            origin_server_ts: raw.origin_server_ts,
            prev_content: raw.prev_content.map(|prev| PowerLevelsEventContent {
                ban: prev.ban,
                events: prev.events,
                events_default: prev.events_default,
                invite: prev.invite,
                kick: prev.kick,
                redact: prev.redact,
                state_default: prev.state_default,
                users: prev.users,
                users_default: prev.users_default,
                notifications: prev.notifications,
            }),
            room_id: raw.room_id,
            sender: raw.sender,
            state_key: raw.state_key,
            unsigned: raw.unsigned,
        })
    }
}

impl<'a> TryFrom<&'a str> for PowerLevelsEvent {
    type Error = InvalidEvent;

    /// Attempt to create `Self` from parsing a string of JSON data.
    fn try_from(json: &'a str) -> Result<Self, Self::Error> {
        FromStr::from_str(json)
    }
}

impl Serialize for PowerLevelsEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut len = 6;

        if self.prev_content.is_some() {
            len += 1;
        }

        if self.room_id.is_some() {
            len += 1;
        }

        if self.unsigned.is_some() {
            len += 1;
        }

        let mut state = serializer.serialize_struct("PowerLevelsEvent", len)?;

        state.serialize_field("content", &self.content)?;
        state.serialize_field("event_id", &self.event_id)?;
        state.serialize_field("origin_server_ts", &self.origin_server_ts)?;

        if self.prev_content.is_some() {
            state.serialize_field("prev_content", &self.prev_content)?;
        }

        if self.room_id.is_some() {
            state.serialize_field("room_id", &self.room_id)?;
        }

        state.serialize_field("sender", &self.sender)?;
        state.serialize_field("state_key", &self.state_key)?;
        state.serialize_field("type", &self.event_type())?;

        if self.unsigned.is_some() {
            state.serialize_field("unsigned", &self.unsigned)?;
        }

        state.end()
    }
}

impl_state_event!(
    PowerLevelsEvent,
    PowerLevelsEventContent,
    EventType::RoomPowerLevels
);

impl FromStr for PowerLevelsEventContent {
    type Err = InvalidEvent;

    /// Attempt to create `Self` from parsing a string of JSON data.
    fn from_str(json: &str) -> Result<Self, Self::Err> {
        let raw = match serde_json::from_str::<raw::PowerLevelsEventContent>(json) {
            Ok(raw) => raw,
            Err(error) => match serde_json::from_str::<serde_json::Value>(json) {
                Ok(value) => {
                    return Err(InvalidEvent(InnerInvalidEvent::Validation {
                        json: value,
                        message: error.to_string(),
                    }));
                }
                Err(error) => {
                    return Err(InvalidEvent(InnerInvalidEvent::Deserialization { error }));
                }
            },
        };

        Ok(Self {
            ban: raw.ban,
            events: raw.events,
            events_default: raw.events_default,
            invite: raw.invite,
            kick: raw.kick,
            redact: raw.redact,
            state_default: raw.state_default,
            users: raw.users,
            users_default: raw.users_default,
            notifications: raw.notifications,
        })
    }
}

impl<'a> TryFrom<&'a str> for PowerLevelsEventContent {
    type Error = InvalidEvent;

    /// Attempt to create `Self` from parsing a string of JSON data.
    fn try_from(json: &'a str) -> Result<Self, Self::Error> {
        FromStr::from_str(json)
    }
}

mod raw {
    use super::*;

    /// Defines the power levels (privileges) of users in the room.
    #[derive(Clone, Debug, Deserialize, PartialEq)]
    pub struct PowerLevelsEvent {
        /// The event's content.
        pub content: PowerLevelsEventContent,

        /// The unique identifier for the event.
        pub event_id: EventId,

        /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver when this
        /// event was sent.
        pub origin_server_ts: UInt,

        /// The previous content for this state key, if any.
        pub prev_content: Option<PowerLevelsEventContent>,

        /// The unique identifier for the room associated with this event.
        pub room_id: Option<RoomId>,

        /// Additional key-value pairs not signed by the homeserver.
        pub unsigned: Option<Value>,

        /// The unique identifier for the user who sent this event.
        pub sender: UserId,

        /// A key that determines which piece of room state the event represents.
        pub state_key: String,
    }

    /// The payload for `PowerLevelsEvent`.
    #[derive(Clone, Debug, Deserialize, PartialEq)]
    pub struct PowerLevelsEventContent {
        /// The level required to ban a user.
        #[serde(default = "default_power_level")]
        pub ban: Int,

        /// The level required to send specific event types.
        ///
        /// This is a mapping from event type to power level required.
        pub events: HashMap<EventType, Int>,

        /// The default level required to send message events.
        #[serde(default)]
        pub events_default: Int,

        /// The level required to invite a user.
        #[serde(default = "default_power_level")]
        pub invite: Int,

        /// The level required to kick a user.
        #[serde(default = "default_power_level")]
        pub kick: Int,

        /// The level required to redact an event.
        #[serde(default = "default_power_level")]
        pub redact: Int,

        /// The default level required to send state events.
        #[serde(default = "default_power_level")]
        pub state_default: Int,

        /// The power levels for specific users.
        ///
        /// This is a mapping from `user_id` to power level for that user.
        pub users: HashMap<UserId, Int>,

        /// The default power level for every user in the room.
        #[serde(default)]
        pub users_default: Int,

        /// The power level requirements for specific notification types.
        ///
        /// This is a mapping from `key` to power level for that notifications key.
        pub notifications: NotificationPowerLevels,
    }
}

/// The power level requirements for specific notification types.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct NotificationPowerLevels {
    /// The level required to trigger an `@room` notification.
    #[serde(default = "default_power_level")]
    pub room: Int,
}

/// Used to default power levels to 50 during deserialization.
fn default_power_level() -> Int {
    Int::from(50)
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, convert::TryFrom};

    use js_int::{Int, UInt};
    use ruma_identifiers::{EventId, RoomId, UserId};
    use serde_json::Value;

    use super::{NotificationPowerLevels, PowerLevelsEvent, PowerLevelsEventContent};

    #[test]
    fn serialization_with_optional_fields_as_none() {
        let default = Int::try_from(50).unwrap();

        let power_levels_event = PowerLevelsEvent {
            content: PowerLevelsEventContent {
                ban: default,
                events: HashMap::new(),
                events_default: default,
                invite: default,
                kick: default,
                redact: default,
                state_default: default,
                users: HashMap::new(),
                users_default: default,
                notifications: NotificationPowerLevels { room: default },
            },
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UInt::try_from(1).unwrap(),
            prev_content: None,
            room_id: None,
            unsigned: None,
            sender: UserId::try_from("@carl:example.com").unwrap(),
            state_key: "".to_string(),
        };

        let actual = serde_json::to_string(&power_levels_event).unwrap();
        let expected = r#"{"content":{"ban":50,"events":{},"events_default":50,"invite":50,"kick":50,"redact":50,"state_default":50,"users":{},"users_default":50,"notifications":{"room":50}},"event_id":"$h29iv0s8:example.com","origin_server_ts":1,"sender":"@carl:example.com","state_key":"","type":"m.room.power_levels"}"#;

        assert_eq!(actual, expected);
    }

    #[test]
    fn serialization_with_all_fields() {
        let default = Int::try_from(50).unwrap();

        let power_levels_event = PowerLevelsEvent {
            content: PowerLevelsEventContent {
                ban: default,
                events: HashMap::new(),
                events_default: default,
                invite: default,
                kick: default,
                redact: default,
                state_default: default,
                users: HashMap::new(),
                users_default: default,
                notifications: NotificationPowerLevels { room: default },
            },
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UInt::try_from(1).unwrap(),
            prev_content: Some(PowerLevelsEventContent {
                // Make just one field different so we at least know they're two different objects.
                ban: Int::try_from(75).unwrap(),
                events: HashMap::new(),
                events_default: default,
                invite: default,
                kick: default,
                redact: default,
                state_default: default,
                users: HashMap::new(),
                users_default: default,
                notifications: NotificationPowerLevels { room: default },
            }),
            room_id: Some(RoomId::try_from("!n8f893n9:example.com").unwrap()),
            unsigned: Some(serde_json::from_str::<Value>(r#"{"foo":"bar"}"#).unwrap()),
            sender: UserId::try_from("@carl:example.com").unwrap(),
            state_key: "".to_string(),
        };

        let actual = serde_json::to_string(&power_levels_event).unwrap();
        let expected = r#"{"content":{"ban":50,"events":{},"events_default":50,"invite":50,"kick":50,"redact":50,"state_default":50,"users":{},"users_default":50,"notifications":{"room":50}},"event_id":"$h29iv0s8:example.com","origin_server_ts":1,"prev_content":{"ban":75,"events":{},"events_default":50,"invite":50,"kick":50,"redact":50,"state_default":50,"users":{},"users_default":50,"notifications":{"room":50}},"room_id":"!n8f893n9:example.com","sender":"@carl:example.com","state_key":"","type":"m.room.power_levels","unsigned":{"foo":"bar"}}"#;

        assert_eq!(actual, expected);
    }
}
