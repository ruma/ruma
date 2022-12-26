//! Types for the [`m.room.power_levels`] event.
//!
//! [`m.room.power_levels`]: https://spec.matrix.org/v1.4/client-server-api/#mroompower_levels

use std::{cmp::max, collections::BTreeMap};

use js_int::{int, Int};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{
    events::{EmptyStateKey, MessageLikeEventType, StateEventType, TimelineEventType},
    power_levels::{default_power_level, NotificationPowerLevels},
    OwnedUserId, UserId,
};

/// The content of an `m.room.power_levels` event.
///
/// Defines the power levels (privileges) of users in the room.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.power_levels", kind = State, state_key_type = EmptyStateKey)]
pub struct RoomPowerLevelsEventContent {
    /// The level required to ban a user.
    #[serde(
        default = "default_power_level",
        skip_serializing_if = "is_default_power_level",
        deserialize_with = "crate::serde::deserialize_v1_powerlevel"
    )]
    #[ruma_event(skip_redaction)]
    pub ban: Int,

    /// The level required to send specific event types.
    ///
    /// This is a mapping from event type to power level required.
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "crate::serde::btreemap_deserialize_v1_powerlevel_values"
    )]
    #[ruma_event(skip_redaction)]
    pub events: BTreeMap<TimelineEventType, Int>,

    /// The default level required to send message events.
    #[serde(
        default,
        skip_serializing_if = "crate::serde::is_default",
        deserialize_with = "crate::serde::deserialize_v1_powerlevel"
    )]
    #[ruma_event(skip_redaction)]
    pub events_default: Int,

    /// The level required to invite a user.
    #[serde(
        default,
        skip_serializing_if = "crate::serde::is_default",
        deserialize_with = "crate::serde::deserialize_v1_powerlevel"
    )]
    pub invite: Int,

    /// The level required to kick a user.
    #[serde(
        default = "default_power_level",
        skip_serializing_if = "is_default_power_level",
        deserialize_with = "crate::serde::deserialize_v1_powerlevel"
    )]
    #[ruma_event(skip_redaction)]
    pub kick: Int,

    /// The level required to redact an event.
    #[serde(
        default = "default_power_level",
        skip_serializing_if = "is_default_power_level",
        deserialize_with = "crate::serde::deserialize_v1_powerlevel"
    )]
    #[ruma_event(skip_redaction)]
    pub redact: Int,

    /// The default level required to send state events.
    #[serde(
        default = "default_power_level",
        skip_serializing_if = "is_default_power_level",
        deserialize_with = "crate::serde::deserialize_v1_powerlevel"
    )]
    #[ruma_event(skip_redaction)]
    pub state_default: Int,

    /// The power levels for specific users.
    ///
    /// This is a mapping from `user_id` to power level for that user.
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "crate::serde::btreemap_deserialize_v1_powerlevel_values"
    )]
    #[ruma_event(skip_redaction)]
    pub users: BTreeMap<OwnedUserId, Int>,

    /// The default power level for every user in the room.
    #[serde(
        default,
        skip_serializing_if = "crate::serde::is_default",
        deserialize_with = "crate::serde::deserialize_v1_powerlevel"
    )]
    #[ruma_event(skip_redaction)]
    pub users_default: Int,

    /// The power level requirements for specific notification types.
    ///
    /// This is a mapping from `key` to power level for that notifications key.
    #[serde(default, skip_serializing_if = "NotificationPowerLevels::is_default")]
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

impl SyncRoomPowerLevelsEvent {
    /// Obtain the effective power levels, regardless of whether this event is redacted.
    pub fn power_levels(&self) -> RoomPowerLevels {
        match self {
            Self::Original(ev) => ev.content.clone().into(),
            Self::Redacted(ev) => ev.content.clone().into(),
        }
    }
}

impl StrippedRoomPowerLevelsEvent {
    /// Obtain the effective power levels from this event.
    pub fn power_levels(&self) -> RoomPowerLevels {
        self.content.clone().into()
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
    pub ban: Int,

    /// The level required to send specific event types.
    ///
    /// This is a mapping from event type to power level required.
    pub events: BTreeMap<TimelineEventType, Int>,

    /// The default level required to send message events.
    pub events_default: Int,

    /// The level required to invite a user.
    pub invite: Int,

    /// The level required to kick a user.
    pub kick: Int,

    /// The level required to redact an event.
    pub redact: Int,

    /// The default level required to send state events.
    pub state_default: Int,

    /// The power levels for specific users.
    ///
    /// This is a mapping from `user_id` to power level for that user.
    pub users: BTreeMap<OwnedUserId, Int>,

    /// The default power level for every user in the room.
    pub users_default: Int,

    /// The power level requirements for specific notification types.
    ///
    /// This is a mapping from `key` to power level for that notifications key.
    pub notifications: NotificationPowerLevels,
}

impl RoomPowerLevels {
    /// Get the power level of a specific user.
    pub fn for_user(&self, user_id: &UserId) -> Int {
        self.users.get(user_id).map_or(self.users_default, |pl| *pl)
    }

    /// Whether the given user can do the given action based on the power levels.
    pub fn user_can_do(&self, user_id: &UserId, action: PowerLevelAction) -> bool {
        let user_pl = self.for_user(user_id);

        match action {
            PowerLevelAction::Ban => user_pl >= self.ban,
            PowerLevelAction::Invite => user_pl >= self.invite,
            PowerLevelAction::Kick => user_pl >= self.kick,
            PowerLevelAction::Redact => user_pl >= self.redact,
            PowerLevelAction::SendMessage(message_type) => {
                user_pl
                    >= self
                        .events
                        .get(&message_type.into())
                        .map(ToOwned::to_owned)
                        .unwrap_or(self.events_default)
            }
            PowerLevelAction::SendState(state_type) => {
                user_pl
                    >= self
                        .events
                        .get(&state_type.into())
                        .map(ToOwned::to_owned)
                        .unwrap_or(self.state_default)
            }
            PowerLevelAction::TriggerNotification(notification_type) => match notification_type {
                NotificationPowerLevelType::Room => user_pl >= self.notifications.room,
            },
        }
    }

    /// Get the maximum power level of any user.
    pub fn max(&self) -> Int {
        self.users.values().fold(self.users_default, |max_pl, user_pl| max(max_pl, *user_pl))
    }
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

impl From<RoomPowerLevels> for RoomPowerLevelsEventContent {
    fn from(c: RoomPowerLevels) -> Self {
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

/// The actions that can be limited by power levels.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PowerLevelAction {
    /// Ban a user.
    Ban,

    /// Invite a user.
    Invite,

    /// Kick a user.
    Kick,

    /// Redact an event.
    Redact,

    /// Send a message-like event.
    SendMessage(MessageLikeEventType),

    /// Send a state event.
    SendState(StateEventType),

    /// Trigger a notification.
    TriggerNotification(NotificationPowerLevelType),
}

/// The notification types that can be limited by power levels.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum NotificationPowerLevelType {
    /// `@room` notifications.
    Room,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use assign::assign;
    use js_int::int;
    use maplit::btreemap;
    use serde_json::{json, to_value as to_json_value};

    use super::{default_power_level, NotificationPowerLevels, RoomPowerLevelsEventContent};
    use crate::user_id;

    #[test]
    fn serialization_with_optional_fields_as_none() {
        let default = default_power_level();

        let power_levels = RoomPowerLevelsEventContent {
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
        };

        let actual = to_json_value(&power_levels).unwrap();
        let expected = json!({});

        assert_eq!(actual, expected);
    }

    #[test]
    fn serialization_with_all_fields() {
        let user = user_id!("@carl:example.com");
        let power_levels_event = RoomPowerLevelsEventContent {
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
        };

        let actual = to_json_value(&power_levels_event).unwrap();
        let expected = json!({
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
            },
        });

        assert_eq!(actual, expected);
    }
}
