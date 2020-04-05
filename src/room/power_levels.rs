//! Types for the *m.room.power_levels* event.

use std::collections::HashMap;

use js_int::{Int, UInt};
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{EventType, FromRaw};

/// Defines the power levels (privileges) of users in the room.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename = "m.room.power_levels", tag = "type")]
pub struct PowerLevelsEvent {
    /// The event's content.
    pub content: PowerLevelsEventContent,

    /// The unique identifier for the event.
    pub event_id: EventId,

    /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver when this
    /// event was sent.
    pub origin_server_ts: UInt,

    /// The previous content for this state key, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_content: Option<PowerLevelsEventContent>,

    /// The unique identifier for the room associated with this event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_id: Option<RoomId>,

    /// Additional key-value pairs not signed by the homeserver.
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub unsigned: Map<String, Value>,

    /// The unique identifier for the user who sent this event.
    pub sender: UserId,

    /// A key that determines which piece of room state the event represents.
    pub state_key: String,
}

/// The payload for `PowerLevelsEvent`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PowerLevelsEventContent {
    /// The level required to ban a user.
    #[serde(skip_serializing_if = "is_default_power_level")]
    pub ban: Int,

    /// The level required to send specific event types.
    ///
    /// This is a mapping from event type to power level required.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub events: HashMap<EventType, Int>,

    /// The default level required to send message events.
    #[serde(skip_serializing_if = "is_power_level_zero")]
    pub events_default: Int,

    /// The level required to invite a user.
    #[serde(skip_serializing_if = "is_default_power_level")]
    pub invite: Int,

    /// The level required to kick a user.
    #[serde(skip_serializing_if = "is_default_power_level")]
    pub kick: Int,

    /// The level required to redact an event.
    #[serde(skip_serializing_if = "is_default_power_level")]
    pub redact: Int,

    /// The default level required to send state events.
    #[serde(skip_serializing_if = "is_default_power_level")]
    pub state_default: Int,

    /// The power levels for specific users.
    ///
    /// This is a mapping from `user_id` to power level for that user.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub users: HashMap<UserId, Int>,

    /// The default power level for every user in the room.
    #[serde(skip_serializing_if = "is_power_level_zero")]
    pub users_default: Int,

    /// The power level requirements for specific notification types.
    ///
    /// This is a mapping from `key` to power level for that notifications key.
    #[serde(skip_serializing_if = "NotificationPowerLevels::is_default")]
    pub notifications: NotificationPowerLevels,
}

impl FromRaw for PowerLevelsEvent {
    type Raw = raw::PowerLevelsEvent;

    fn from_raw(raw: raw::PowerLevelsEvent) -> Self {
        Self {
            content: FromRaw::from_raw(raw.content),
            event_id: raw.event_id,
            origin_server_ts: raw.origin_server_ts,
            prev_content: raw.prev_content.map(FromRaw::from_raw),
            room_id: raw.room_id,
            unsigned: raw.unsigned,
            sender: raw.sender,
            state_key: raw.state_key,
        }
    }
}

impl FromRaw for PowerLevelsEventContent {
    type Raw = raw::PowerLevelsEventContent;

    fn from_raw(raw: raw::PowerLevelsEventContent) -> Self {
        Self {
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
        }
    }
}

impl_state_event!(
    PowerLevelsEvent,
    PowerLevelsEventContent,
    EventType::RoomPowerLevels
);

pub(crate) mod raw {
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
        #[serde(default)]
        pub unsigned: Map<String, Value>,

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
        #[serde(default)]
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
        #[serde(default)]
        pub users: HashMap<UserId, Int>,

        /// The default power level for every user in the room.
        #[serde(default)]
        pub users_default: Int,

        /// The power level requirements for specific notification types.
        ///
        /// This is a mapping from `key` to power level for that notifications key.
        #[serde(default)]
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

impl NotificationPowerLevels {
    // TODO: Make public under this name?
    // pass-by-ref required for #[serde(skip_serializing_if)]
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

impl Default for NotificationPowerLevels {
    fn default() -> Self {
        Self {
            room: default_power_level(),
        }
    }
}

/// Used to default power levels to 50 during deserialization.
fn default_power_level() -> Int {
    Int::from(50)
}

/// Used with #[serde(skip_serializing_if)] to omit default power levels.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_default_power_level(l: &Int) -> bool {
    *l == Int::from(50)
}

/// Used with #[serde(skip_serializing_if)] to omit default power levels.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_power_level_zero(l: &Int) -> bool {
    *l == Int::from(0)
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, convert::TryFrom};

    use js_int::{Int, UInt};
    use maplit::hashmap;
    use ruma_identifiers::{EventId, RoomId, UserId};
    use serde_json::{json, to_value as to_json_value, Map};

    use super::{
        default_power_level, NotificationPowerLevels, PowerLevelsEvent, PowerLevelsEventContent,
    };
    use crate::EventType;

    #[test]
    fn serialization_with_optional_fields_as_none() {
        let default = default_power_level();

        let power_levels_event = PowerLevelsEvent {
            content: PowerLevelsEventContent {
                ban: default,
                events: HashMap::new(),
                events_default: Int::from(0),
                invite: default,
                kick: default,
                redact: default,
                state_default: default,
                users: HashMap::new(),
                users_default: Int::from(0),
                notifications: NotificationPowerLevels::default(),
            },
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UInt::from(1u32),
            prev_content: None,
            room_id: None,
            unsigned: Map::new(),
            sender: UserId::try_from("@carl:example.com").unwrap(),
            state_key: "".to_string(),
        };

        let actual = to_json_value(&power_levels_event).unwrap();
        let expected = json!({
            "content": {},
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.power_levels"
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn serialization_with_all_fields() {
        let user = UserId::try_from("@carl:example.com").unwrap();
        let power_levels_event = PowerLevelsEvent {
            content: PowerLevelsEventContent {
                ban: Int::from(23),
                events: hashmap! {
                    EventType::Dummy => Int::from(23)
                },
                events_default: Int::from(23),
                invite: Int::from(23),
                kick: Int::from(23),
                redact: Int::from(23),
                state_default: Int::from(23),
                users: hashmap! {
                    user.clone() => Int::from(23)
                },
                users_default: Int::from(23),
                notifications: NotificationPowerLevels {
                    room: Int::from(23),
                },
            },
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UInt::from(1u32),
            prev_content: Some(PowerLevelsEventContent {
                // Make just one field different so we at least know they're two different objects.
                ban: Int::from(42),
                events: hashmap! {
                    EventType::Dummy => Int::from(42)
                },
                events_default: Int::from(42),
                invite: Int::from(42),
                kick: Int::from(42),
                redact: Int::from(42),
                state_default: Int::from(42),
                users: hashmap! {
                    user.clone() => Int::from(42)
                },
                users_default: Int::from(42),
                notifications: NotificationPowerLevels {
                    room: Int::from(42),
                },
            }),
            room_id: Some(RoomId::try_from("!n8f893n9:example.com").unwrap()),
            unsigned: serde_json::from_str(r#"{"foo": "bar"}"#).unwrap(),
            sender: user,
            state_key: "".to_string(),
        };

        let actual = to_json_value(&power_levels_event).unwrap();
        let expected = json!({
            "content": {
                "ban": 23,
                "events": {
                    "m.dummy": 23
                },
                "events_default": 23,
                "invite": 23,
                "kick": 23,
                "redact": 23,
                "state_default": 23,
                "users": {
                    "@carl:example.com": 23
                },
                "users_default": 23,
                "notifications": {
                    "room": 23
                }
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "prev_content": {
                "ban": 42,
                "events": {
                    "m.dummy": 42
                },
                "events_default": 42,
                "invite": 42,
                "kick": 42,
                "redact": 42,
                "state_default": 42,
                "users": {
                    "@carl:example.com": 42
                },
                "users_default": 42,
                "notifications": {
                    "room": 42
                }
            },
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.power_levels",
            "unsigned": {
                "foo": "bar"
            }
        });

        assert_eq!(actual, expected);
    }
}
