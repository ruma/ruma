//! Types for the [`m.room.power_levels`] event.
//!
//! [`m.room.power_levels`]: https://spec.matrix.org/v1.2/client-server-api/#mroompower_levels

use std::collections::BTreeMap;

use js_int::{int, Int};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{
    events::RoomEventType,
    power_levels::{default_power_level, NotificationPowerLevels},
    UserId,
};

/// The content of an `m.room.power_levels` event.
///
/// Defines the power levels (privileges) of users in the room.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.power_levels", kind = State)]
pub struct RoomPowerLevelsEventContent {
    /// The level required to ban a user.
    ///
    /// If you activate the `compat` feature, deserialization will work for stringified
    /// integers too.
    #[cfg_attr(
        feature = "compat",
        serde(deserialize_with = "crate::serde::deserialize_v1_powerlevel")
    )]
    #[serde(default = "default_power_level", skip_serializing_if = "is_default_power_level")]
    #[ruma_event(skip_redaction)]
    pub ban: Int,

    /// The level required to send specific event types.
    ///
    /// This is a mapping from event type to power level required.
    ///
    /// If you activate the `compat` feature, deserialization will work for stringified
    /// integers too.
    #[cfg_attr(
        feature = "compat",
        serde(deserialize_with = "crate::serde::btreemap_deserialize_v1_powerlevel_values")
    )]
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[ruma_event(skip_redaction)]
    pub events: BTreeMap<RoomEventType, Int>,

    /// The default level required to send message events.
    ///
    /// If you activate the `compat` feature, deserialization will work for stringified
    /// integers too.
    #[cfg_attr(
        feature = "compat",
        serde(deserialize_with = "crate::serde::deserialize_v1_powerlevel")
    )]
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    #[ruma_event(skip_redaction)]
    pub events_default: Int,

    /// The level required to invite a user.
    ///
    /// If you activate the `compat` feature, deserialization will work for stringified
    /// integers too.
    #[cfg_attr(
        feature = "compat",
        serde(deserialize_with = "crate::serde::deserialize_v1_powerlevel")
    )]
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub invite: Int,

    /// The level required to kick a user.
    ///
    /// If you activate the `compat` feature, deserialization will work for stringified
    /// integers too.
    #[cfg_attr(
        feature = "compat",
        serde(deserialize_with = "crate::serde::deserialize_v1_powerlevel")
    )]
    #[serde(default = "default_power_level", skip_serializing_if = "is_default_power_level")]
    #[ruma_event(skip_redaction)]
    pub kick: Int,

    /// The level required to redact an event.
    ///
    /// If you activate the `compat` feature, deserialization will work for stringified
    /// integers too.
    #[cfg_attr(
        feature = "compat",
        serde(deserialize_with = "crate::serde::deserialize_v1_powerlevel")
    )]
    #[serde(default = "default_power_level", skip_serializing_if = "is_default_power_level")]
    #[ruma_event(skip_redaction)]
    pub redact: Int,

    /// The default level required to send state events.
    ///
    /// If you activate the `compat` feature, deserialization will work for stringified
    /// integers too.
    #[cfg_attr(
        feature = "compat",
        serde(deserialize_with = "crate::serde::deserialize_v1_powerlevel")
    )]
    #[serde(default = "default_power_level", skip_serializing_if = "is_default_power_level")]
    #[ruma_event(skip_redaction)]
    pub state_default: Int,

    /// The power levels for specific users.
    ///
    /// This is a mapping from `user_id` to power level for that user.
    ///
    /// If you activate the `compat` feature, deserialization will work for stringified
    /// integers too.
    #[cfg_attr(
        feature = "compat",
        serde(deserialize_with = "crate::serde::btreemap_deserialize_v1_powerlevel_values")
    )]
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[ruma_event(skip_redaction)]
    pub users: BTreeMap<Box<UserId>, Int>,

    /// The default power level for every user in the room.
    ///
    /// If you activate the `compat` feature, deserialization will work for stringified
    /// integers too.
    #[cfg_attr(
        feature = "compat",
        serde(deserialize_with = "crate::serde::deserialize_v1_powerlevel")
    )]
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    #[ruma_event(skip_redaction)]
    pub users_default: Int,

    /// The power level requirements for specific notification types.
    ///
    /// This is a mapping from `key` to power level for that notifications key.
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub notifications: NotificationPowerLevels,
}

impl RoomPowerLevelsEventContent {
    /// Creates a new `RoomPowerLevelsEventContent` with all-default values.
    pub fn new() -> Self {
        // events_default, users_default and invite having a default of 0 while the others have a
        // default of 50 is not an oversight, these defaults are from the Matrix specification.
        Self {
            ban: default_power_level(),
            events: BTreeMap::new(),
            events_default: int!(0),
            invite: int!(0),
            kick: default_power_level(),
            redact: default_power_level(),
            state_default: default_power_level(),
            users: BTreeMap::new(),
            users_default: int!(0),
            notifications: NotificationPowerLevels::default(),
        }
    }
}

impl Default for RoomPowerLevelsEventContent {
    fn default() -> Self {
        Self::new()
    }
}

/// Used with `#[serde(skip_serializing_if)]` to omit default power levels.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_default_power_level(l: &Int) -> bool {
    *l == int!(50)
}

impl RoomPowerLevelsEvent {
    /// Obtain the effective power levels, regardless of whether this event is redacted.
    pub fn power_levels(&self) -> RoomPowerLevels {
        match self {
            Self::Original(ev) => ev.content.clone().into(),
            Self::Redacted(ev) => ev.content.clone().into(),
        }
    }
}

/// The effective power levels of a room.
///
/// This struct contains the same fields as [`RoomPowerLevelsEventContent`] and be created from that
/// using a `From` trait implementation, but it is also implements
/// `From<`[`RedactedRoomPowerLevelsEventContent`]`>`, so can be used when wanting to inspect the
/// power levels of a room, regardless of whether the most recent power-levels event is redacted or
/// not.
#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RoomPowerLevels {
    /// The level required to ban a user.
    ///
    /// If you activate the `compat` feature, deserialization will work for stringified
    /// integers too.
    pub ban: Int,

    /// The level required to send specific event types.
    ///
    /// This is a mapping from event type to power level required.
    ///
    /// If you activate the `compat` feature, deserialization will work for stringified
    /// integers too.
    pub events: BTreeMap<RoomEventType, Int>,

    /// The default level required to send message events.
    ///
    /// If you activate the `compat` feature, deserialization will work for stringified
    /// integers too.
    pub events_default: Int,

    /// The level required to invite a user.
    ///
    /// If you activate the `compat` feature, deserialization will work for stringified
    /// integers too.
    pub invite: Int,

    /// The level required to kick a user.
    ///
    /// If you activate the `compat` feature, deserialization will work for stringified
    /// integers too.
    pub kick: Int,

    /// The level required to redact an event.
    ///
    /// If you activate the `compat` feature, deserialization will work for stringified
    /// integers too.
    pub redact: Int,

    /// The default level required to send state events.
    ///
    /// If you activate the `compat` feature, deserialization will work for stringified
    /// integers too.
    pub state_default: Int,

    /// The power levels for specific users.
    ///
    /// This is a mapping from `user_id` to power level for that user.
    ///
    /// If you activate the `compat` feature, deserialization will work for stringified
    /// integers too.
    pub users: BTreeMap<Box<UserId>, Int>,

    /// The default power level for every user in the room.
    ///
    /// If you activate the `compat` feature, deserialization will work for stringified
    /// integers too.
    pub users_default: Int,

    /// The power level requirements for specific notification types.
    ///
    /// This is a mapping from `key` to power level for that notifications key.
    pub notifications: NotificationPowerLevels,
}

impl From<RoomPowerLevelsEventContent> for RoomPowerLevels {
    fn from(c: RoomPowerLevelsEventContent) -> Self {
        Self {
            ban: c.ban,
            events: c.events,
            events_default: c.events_default,
            invite: c.invite,
            kick: c.kick,
            redact: c.redact,
            state_default: c.state_default,
            users: c.users,
            users_default: c.users_default,
            notifications: c.notifications,
        }
    }
}

impl From<RedactedRoomPowerLevelsEventContent> for RoomPowerLevels {
    fn from(c: RedactedRoomPowerLevelsEventContent) -> Self {
        Self {
            ban: c.ban,
            events: c.events,
            events_default: c.events_default,
            invite: int!(0),
            kick: c.kick,
            redact: c.redact,
            state_default: c.state_default,
            users: c.users,
            users_default: c.users_default,
            notifications: NotificationPowerLevels::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{event_id, room_id, user_id, MilliSecondsSinceUnixEpoch};
    use assign::assign;
    use js_int::{int, uint};
    use maplit::btreemap;
    use serde_json::{json, to_value as to_json_value};

    use super::{default_power_level, NotificationPowerLevels, RoomPowerLevelsEventContent};
    use crate::events::{OriginalStateEvent, StateUnsigned};

    #[test]
    fn serialization_with_optional_fields_as_none() {
        let default = default_power_level();

        let power_levels_event = OriginalStateEvent {
            content: RoomPowerLevelsEventContent {
                ban: default,
                events: BTreeMap::new(),
                events_default: int!(0),
                invite: int!(0),
                kick: default,
                redact: default,
                state_default: default,
                users: BTreeMap::new(),
                users_default: int!(0),
                notifications: NotificationPowerLevels::default(),
            },
            event_id: event_id!("$h29iv0s8:example.com").to_owned(),
            origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
            room_id: room_id!("!n8f893n9:example.com").to_owned(),
            unsigned: StateUnsigned::default(),
            sender: user_id!("@carl:example.com").to_owned(),
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
        let power_levels_event = OriginalStateEvent {
            content: RoomPowerLevelsEventContent {
                ban: int!(23),
                events: btreemap! {
                    "m.dummy".into() => int!(23)
                },
                events_default: int!(23),
                invite: int!(23),
                kick: int!(23),
                redact: int!(23),
                state_default: int!(23),
                users: btreemap! {
                    user.to_owned() => int!(23)
                },
                users_default: int!(23),
                notifications: assign!(NotificationPowerLevels::new(), { room: int!(23) }),
            },
            event_id: event_id!("$h29iv0s8:example.com").to_owned(),
            origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
            room_id: room_id!("!n8f893n9:example.com").to_owned(),
            unsigned: StateUnsigned {
                age: Some(int!(100)),
                prev_content: Some(RoomPowerLevelsEventContent {
                    // Make just one field different so we at least know they're two different
                    // objects.
                    ban: int!(42),
                    events: btreemap! {
                        "m.dummy".into() => int!(42)
                    },
                    events_default: int!(42),
                    invite: int!(42),
                    kick: int!(42),
                    redact: int!(42),
                    state_default: int!(42),
                    users: btreemap! {
                        user.to_owned() => int!(42)
                    },
                    users_default: int!(42),
                    notifications: assign!(NotificationPowerLevels::new(), { room: int!(42) }),
                }),
                ..StateUnsigned::default()
            },
            sender: user.to_owned(),
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
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.power_levels",
            "unsigned": {
                "age": 100,
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
                    },
                },
            }
        });

        assert_eq!(actual, expected);
    }
}
