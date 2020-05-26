//! Types for the *m.room.power_levels* event.

use std::collections::BTreeMap;

use js_int::Int;
use ruma_events_macros::ruma_event;
use ruma_identifiers::UserId;
use serde::{Deserialize, Serialize};

use crate::EventType;

ruma_event! {
    /// Defines the power levels (privileges) of users in the room.
    PowerLevelsEvent {
        kind: StateEvent,
        event_type: "m.room.power_levels",
        content: {
            /// The level required to ban a user.
            #[serde(
                default = "default_power_level",
                skip_serializing_if = "is_default_power_level"
            )]
            pub ban: Int,

            /// The level required to send specific event types.
            ///
            /// This is a mapping from event type to power level required.
            #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
            pub events: BTreeMap<EventType, Int>,

            /// The default level required to send message events.
            #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
            pub events_default: Int,

            /// The level required to invite a user.
            #[serde(
                default = "default_power_level",
                skip_serializing_if = "is_default_power_level"
            )]
            pub invite: Int,

            /// The level required to kick a user.
            #[serde(
                default = "default_power_level",
                skip_serializing_if = "is_default_power_level"
            )]
            pub kick: Int,

            /// The level required to redact an event.
            #[serde(
                default = "default_power_level",
                skip_serializing_if = "is_default_power_level"
            )]
            pub redact: Int,

            /// The default level required to send state events.
            #[serde(
                default = "default_power_level",
                skip_serializing_if = "is_default_power_level"
            )]
            pub state_default: Int,

            /// The power levels for specific users.
            ///
            /// This is a mapping from `user_id` to power level for that user.
            #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
            pub users: BTreeMap<UserId, Int>,

            /// The default power level for every user in the room.
            #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
            pub users_default: Int,

            /// The power level requirements for specific notification types.
            ///
            /// This is a mapping from `key` to power level for that notifications key.
            #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
            pub notifications: NotificationPowerLevels,
        },
    }
}

impl Default for PowerLevelsEventContent {
    fn default() -> Self {
        // events_default and users_default having a default of 0 while the others have a default
        // of 50 is not an oversight, these defaults are from the Matrix specification.
        Self {
            ban: default_power_level(),
            events: BTreeMap::new(),
            events_default: Int::default(),
            invite: default_power_level(),
            kick: default_power_level(),
            redact: default_power_level(),
            state_default: default_power_level(),
            users: BTreeMap::new(),
            users_default: Int::default(),
            notifications: NotificationPowerLevels::default(),
        }
    }
}

/// The power level requirements for specific notification types.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct NotificationPowerLevels {
    /// The level required to trigger an `@room` notification.
    #[serde(default = "default_power_level")]
    pub room: Int,
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

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        convert::TryFrom,
        time::{Duration, UNIX_EPOCH},
    };

    use js_int::Int;
    use maplit::btreemap;
    use ruma_identifiers::{EventId, RoomId, UserId};
    use serde_json::{json, to_value as to_json_value};

    use super::{
        default_power_level, NotificationPowerLevels, PowerLevelsEvent, PowerLevelsEventContent,
    };
    use crate::{EventType, UnsignedData};

    #[test]
    fn serialization_with_optional_fields_as_none() {
        let default = default_power_level();

        let power_levels_event = PowerLevelsEvent {
            content: PowerLevelsEventContent {
                ban: default,
                events: BTreeMap::new(),
                events_default: Int::from(0),
                invite: default,
                kick: default,
                redact: default,
                state_default: default,
                users: BTreeMap::new(),
                users_default: Int::from(0),
                notifications: NotificationPowerLevels::default(),
            },
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
            prev_content: None,
            room_id: None,
            unsigned: UnsignedData::default(),
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
                events: btreemap! {
                    EventType::Dummy => Int::from(23)
                },
                events_default: Int::from(23),
                invite: Int::from(23),
                kick: Int::from(23),
                redact: Int::from(23),
                state_default: Int::from(23),
                users: btreemap! {
                    user.clone() => Int::from(23)
                },
                users_default: Int::from(23),
                notifications: NotificationPowerLevels {
                    room: Int::from(23),
                },
            },
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
            prev_content: Some(PowerLevelsEventContent {
                // Make just one field different so we at least know they're two different objects.
                ban: Int::from(42),
                events: btreemap! {
                    EventType::Dummy => Int::from(42)
                },
                events_default: Int::from(42),
                invite: Int::from(42),
                kick: Int::from(42),
                redact: Int::from(42),
                state_default: Int::from(42),
                users: btreemap! {
                    user.clone() => Int::from(42)
                },
                users_default: Int::from(42),
                notifications: NotificationPowerLevels {
                    room: Int::from(42),
                },
            }),
            room_id: Some(RoomId::try_from("!n8f893n9:example.com").unwrap()),
            unsigned: UnsignedData {
                age: Some(Int::from(100)),
                ..UnsignedData::default()
            },
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
                "age": 100
            }
        });

        assert_eq!(actual, expected);
    }
}
