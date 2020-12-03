//! Types for the *m.room.power_levels* event.

use std::collections::BTreeMap;

use js_int::{int, Int};
use ruma_events_macros::StateEventContent;
use ruma_identifiers::UserId;
use serde::{Deserialize, Serialize};

use crate::{EventType, StateEvent};

/// Defines the power levels (privileges) of users in the room.
pub type PowerLevelsEvent = StateEvent<PowerLevelsEventContent>;

/// The payload for `PowerLevelsEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, StateEventContent)]
#[ruma_event(type = "m.room.power_levels")]
pub struct PowerLevelsEventContent {
    /// The level required to ban a user.
    #[cfg_attr(
        feature = "unstable-synapse-quirks",
        serde(deserialize_with = "ruma_serde::int_or_string_to_int")
    )]
    #[serde(
        deserialize_with = "default_power_level",
        skip_serializing_if = "is_default_power_level"
    )]
    #[ruma_event(skip_redaction)]
    pub ban: Int,

    /// The level required to send specific event types.
    ///
    /// This is a mapping from event type to power level required.
    #[cfg_attr(
        feature = "unstable-synapse-quirks",
        serde(deserialize_with = "ruma_serde::btreemap_int_or_string_to_int_values")
    )]
    #[serde(deserialize_with = "ruma_serde::default", skip_serializing_if = "BTreeMap::is_empty")]
    #[ruma_event(skip_redaction)]
    pub events: BTreeMap<EventType, Int>,

    /// The default level required to send message events.
    #[cfg_attr(
        feature = "unstable-synapse-quirks",
        serde(deserialize_with = "ruma_serde::int_or_string_to_int")
    )]
    #[serde(
        deserialize_with = "ruma_serde::default",
        skip_serializing_if = "ruma_serde::is_default"
    )]
    #[ruma_event(skip_redaction)]
    pub events_default: Int,

    /// The level required to invite a user.
    #[cfg_attr(
        feature = "unstable-synapse-quirks",
        serde(deserialize_with = "ruma_serde::int_or_string_to_int")
    )]
    #[serde(
        deserialize_with = "default_power_level",
        skip_serializing_if = "is_default_power_level"
    )]
    pub invite: Int,

    /// The level required to kick a user.
    #[cfg_attr(
        feature = "unstable-synapse-quirks",
        serde(deserialize_with = "ruma_serde::int_or_string_to_int")
    )]
    #[serde(
        deserialize_with = "default_power_level",
        skip_serializing_if = "is_default_power_level"
    )]
    #[ruma_event(skip_redaction)]
    pub kick: Int,

    /// The level required to redact an event.
    #[cfg_attr(
        feature = "unstable-synapse-quirks",
        serde(deserialize_with = "ruma_serde::int_or_string_to_int")
    )]
    #[serde(
        deserialize_with = "default_power_level",
        skip_serializing_if = "is_default_power_level"
    )]
    #[ruma_event(skip_redaction)]
    pub redact: Int,

    /// The default level required to send state events.
    #[cfg_attr(
        feature = "unstable-synapse-quirks",
        serde(deserialize_with = "ruma_serde::int_or_string_to_int")
    )]
    #[serde(
        deserialize_with = "default_power_level",
        skip_serializing_if = "is_default_power_level"
    )]
    #[ruma_event(skip_redaction)]
    pub state_default: Int,

    /// The power levels for specific users.
    ///
    /// This is a mapping from `user_id` to power level for that user.
    #[cfg_attr(
        feature = "unstable-synapse-quirks",
        serde(deserialize_with = "ruma_serde::btreemap_int_or_string_to_int_values")
    )]
    #[serde(deserialize_with = "ruma_serde::default", skip_serializing_if = "BTreeMap::is_empty")]
    #[ruma_event(skip_redaction)]
    pub users: BTreeMap<UserId, Int>,

    /// The default power level for every user in the room.
    #[cfg_attr(
        feature = "unstable-synapse-quirks",
        serde(deserialize_with = "ruma_serde::int_or_string_to_int")
    )]
    #[serde(
        deserialize_with = "ruma_serde::default",
        skip_serializing_if = "ruma_serde::is_default"
    )]
    #[ruma_event(skip_redaction)]
    pub users_default: Int,

    /// The power level requirements for specific notification types.
    ///
    /// This is a mapping from `key` to power level for that notifications key.
    #[serde(
        deserialize_with = "ruma_serde::default",
        skip_serializing_if = "ruma_serde::is_default"
    )]
    pub notifications: NotificationPowerLevels,
}

impl Default for PowerLevelsEventContent {
    fn default() -> Self {
        // events_default and users_default having a default of 0 while the others have a default
        // of 50 is not an oversight, these defaults are from the Matrix specification.
        Self {
            ban: int!(50),
            events: BTreeMap::new(),
            events_default: int!(0),
            invite: int!(50),
            kick: int!(50),
            redact: int!(50),
            state_default: int!(50),
            users: BTreeMap::new(),
            users_default: int!(0),
            notifications: NotificationPowerLevels::default(),
        }
    }
}

/// The power level requirements for specific notification types.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct NotificationPowerLevels {
    /// The level required to trigger an `@room` notification.
    #[cfg_attr(
        feature = "unstable-synapse-quirks",
        serde(deserialize_with = "ruma_serde::int_or_string_to_int")
    )]
    #[serde(deserialize_with = "default_power_level")]
    pub room: Int,
}

impl Default for NotificationPowerLevels {
    fn default() -> Self {
        Self { room: int!(50) }
    }
}

fn default_power_level<'de, D>(deserializer: D) -> Result<Int, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::deserialize(deserializer)?.unwrap_or_else(|| int!(50)))
}

/// Used with #[serde(skip_serializing_if)] to omit default power levels.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_default_power_level(l: &Int) -> bool {
    *l == int!(50)
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        time::{Duration, UNIX_EPOCH},
    };

    use js_int::int;
    use maplit::btreemap;
    use ruma_identifiers::{event_id, room_id, user_id};
    use serde_json::{json, to_value as to_json_value};

    use super::{NotificationPowerLevels, PowerLevelsEventContent};
    use crate::{EventType, StateEvent, Unsigned};

    #[test]
    fn serialization_with_optional_fields_as_none() {
        let power_levels_event = StateEvent {
            content: PowerLevelsEventContent {
                ban: int!(50),
                events: BTreeMap::new(),
                events_default: int!(0),
                invite: int!(50),
                kick: int!(50),
                redact: int!(50),
                state_default: int!(50),
                users: BTreeMap::new(),
                users_default: int!(0),
                notifications: NotificationPowerLevels::default(),
            },
            event_id: event_id!("$h29iv0s8:example.com"),
            origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
            prev_content: None,
            room_id: room_id!("!n8f893n9:example.com"),
            unsigned: Unsigned::default(),
            sender: user_id!("@carl:example.com"),
            state_key: "".into(),
        };

        let actual = to_json_value(&power_levels_event).unwrap();
        let expected = json!({
            "content": {},
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.power_levels"
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn serialization_with_all_fields() {
        let user = user_id!("@carl:example.com");
        let power_levels_event = StateEvent {
            content: PowerLevelsEventContent {
                ban: int!(23),
                events: btreemap! {
                    EventType::Dummy => int!(23)
                },
                events_default: int!(23),
                invite: int!(23),
                kick: int!(23),
                redact: int!(23),
                state_default: int!(23),
                users: btreemap! {
                    user.clone() => int!(23)
                },
                users_default: int!(23),
                notifications: NotificationPowerLevels { room: int!(23) },
            },
            event_id: event_id!("$h29iv0s8:example.com"),
            origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
            prev_content: Some(PowerLevelsEventContent {
                // Make just one field different so we at least know they're two different objects.
                ban: int!(42),
                events: btreemap! {
                    EventType::Dummy => int!(42)
                },
                events_default: int!(42),
                invite: int!(42),
                kick: int!(42),
                redact: int!(42),
                state_default: int!(42),
                users: btreemap! {
                    user.clone() => int!(42)
                },
                users_default: int!(42),
                notifications: NotificationPowerLevels { room: int!(42) },
            }),
            room_id: room_id!("!n8f893n9:example.com"),
            unsigned: Unsigned { age: Some(int!(100)), ..Unsigned::default() },
            sender: user,
            state_key: "".into(),
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
