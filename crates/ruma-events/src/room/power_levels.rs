//! Types for the [`m.room.power_levels`] event.
//!
//! [`m.room.power_levels`]: https://spec.matrix.org/latest/client-server-api/#mroompower_levels

use std::{cmp::max, collections::BTreeMap};

use js_int::{int, Int};
use ruma_common::{
    power_levels::{default_power_level, NotificationPowerLevels},
    push::PushConditionPowerLevelsCtx,
    OwnedUserId, RoomVersionId, UserId,
};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{
    EmptyStateKey, EventContent, MessageLikeEventType, RedactContent, RedactedStateEventContent,
    StateEventType, StaticEventContent, TimelineEventType,
};

/// The content of an `m.room.power_levels` event.
///
/// Defines the power levels (privileges) of users in the room.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.power_levels", kind = State, state_key_type = EmptyStateKey, custom_redacted)]
pub struct RoomPowerLevelsEventContent {
    /// The level required to ban a user.
    #[serde(
        default = "default_power_level",
        skip_serializing_if = "is_default_power_level",
        deserialize_with = "ruma_common::serde::deserialize_v1_powerlevel"
    )]
    pub ban: Int,

    /// The level required to send specific event types.
    ///
    /// This is a mapping from event type to power level required.
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "ruma_common::serde::btreemap_deserialize_v1_powerlevel_values"
    )]
    pub events: BTreeMap<TimelineEventType, Int>,

    /// The default level required to send message events.
    #[serde(
        default,
        skip_serializing_if = "ruma_common::serde::is_default",
        deserialize_with = "ruma_common::serde::deserialize_v1_powerlevel"
    )]
    pub events_default: Int,

    /// The level required to invite a user.
    #[serde(
        default,
        skip_serializing_if = "ruma_common::serde::is_default",
        deserialize_with = "ruma_common::serde::deserialize_v1_powerlevel"
    )]
    pub invite: Int,

    /// The level required to kick a user.
    #[serde(
        default = "default_power_level",
        skip_serializing_if = "is_default_power_level",
        deserialize_with = "ruma_common::serde::deserialize_v1_powerlevel"
    )]
    pub kick: Int,

    /// The level required to redact an event.
    #[serde(
        default = "default_power_level",
        skip_serializing_if = "is_default_power_level",
        deserialize_with = "ruma_common::serde::deserialize_v1_powerlevel"
    )]
    pub redact: Int,

    /// The default level required to send state events.
    #[serde(
        default = "default_power_level",
        skip_serializing_if = "is_default_power_level",
        deserialize_with = "ruma_common::serde::deserialize_v1_powerlevel"
    )]
    pub state_default: Int,

    /// The power levels for specific users.
    ///
    /// This is a mapping from `user_id` to power level for that user.
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "ruma_common::serde::btreemap_deserialize_v1_powerlevel_values"
    )]
    pub users: BTreeMap<OwnedUserId, Int>,

    /// The default power level for every user in the room.
    #[serde(
        default,
        skip_serializing_if = "ruma_common::serde::is_default",
        deserialize_with = "ruma_common::serde::deserialize_v1_powerlevel"
    )]
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

impl RedactContent for RoomPowerLevelsEventContent {
    type Redacted = RedactedRoomPowerLevelsEventContent;

    fn redact(self, version: &RoomVersionId) -> Self::Redacted {
        let Self {
            ban,
            events,
            events_default,
            invite,
            kick,
            redact,
            state_default,
            users,
            users_default,
            ..
        } = self;

        let invite = match version {
            RoomVersionId::V1
            | RoomVersionId::V2
            | RoomVersionId::V3
            | RoomVersionId::V4
            | RoomVersionId::V5
            | RoomVersionId::V6
            | RoomVersionId::V7
            | RoomVersionId::V8
            | RoomVersionId::V9
            | RoomVersionId::V10 => int!(0),
            _ => invite,
        };

        RedactedRoomPowerLevelsEventContent {
            ban,
            events,
            events_default,
            invite,
            kick,
            redact,
            state_default,
            users,
            users_default,
        }
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

/// Redacted form of [`RoomPowerLevelsEventContent`].
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RedactedRoomPowerLevelsEventContent {
    /// The level required to ban a user.
    #[serde(
        default = "default_power_level",
        skip_serializing_if = "is_default_power_level",
        deserialize_with = "ruma_common::serde::deserialize_v1_powerlevel"
    )]
    pub ban: Int,

    /// The level required to send specific event types.
    ///
    /// This is a mapping from event type to power level required.
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "ruma_common::serde::btreemap_deserialize_v1_powerlevel_values"
    )]
    pub events: BTreeMap<TimelineEventType, Int>,

    /// The default level required to send message events.
    #[serde(
        default,
        skip_serializing_if = "ruma_common::serde::is_default",
        deserialize_with = "ruma_common::serde::deserialize_v1_powerlevel"
    )]
    pub events_default: Int,

    /// The level required to invite a user.
    ///
    /// This field was redacted in room versions 1 through 10. Starting from room version 11 it is
    /// preserved.
    #[serde(
        default,
        skip_serializing_if = "ruma_common::serde::is_default",
        deserialize_with = "ruma_common::serde::deserialize_v1_powerlevel"
    )]
    pub invite: Int,

    /// The level required to kick a user.
    #[serde(
        default = "default_power_level",
        skip_serializing_if = "is_default_power_level",
        deserialize_with = "ruma_common::serde::deserialize_v1_powerlevel"
    )]
    pub kick: Int,

    /// The level required to redact an event.
    #[serde(
        default = "default_power_level",
        skip_serializing_if = "is_default_power_level",
        deserialize_with = "ruma_common::serde::deserialize_v1_powerlevel"
    )]
    pub redact: Int,

    /// The default level required to send state events.
    #[serde(
        default = "default_power_level",
        skip_serializing_if = "is_default_power_level",
        deserialize_with = "ruma_common::serde::deserialize_v1_powerlevel"
    )]
    pub state_default: Int,

    /// The power levels for specific users.
    ///
    /// This is a mapping from `user_id` to power level for that user.
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "ruma_common::serde::btreemap_deserialize_v1_powerlevel_values"
    )]
    pub users: BTreeMap<OwnedUserId, Int>,

    /// The default power level for every user in the room.
    #[serde(
        default,
        skip_serializing_if = "ruma_common::serde::is_default",
        deserialize_with = "ruma_common::serde::deserialize_v1_powerlevel"
    )]
    pub users_default: Int,
}

impl EventContent for RedactedRoomPowerLevelsEventContent {
    type EventType = StateEventType;

    fn event_type(&self) -> Self::EventType {
        StateEventType::RoomPowerLevels
    }
}

impl StaticEventContent for RedactedRoomPowerLevelsEventContent {
    const TYPE: &'static str = "m.room.power_levels";
}

impl RedactedStateEventContent for RedactedRoomPowerLevelsEventContent {
    type StateKey = EmptyStateKey;
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

    /// Get the power level required to perform a given action.
    pub fn for_action(&self, action: PowerLevelAction) -> Int {
        match action {
            PowerLevelAction::Ban => self.ban,
            PowerLevelAction::Unban => self.ban.max(self.kick),
            PowerLevelAction::Invite => self.invite,
            PowerLevelAction::Kick => self.kick,
            PowerLevelAction::RedactOwn => self.for_message(MessageLikeEventType::RoomRedaction),
            PowerLevelAction::RedactOther => {
                self.redact.max(self.for_message(MessageLikeEventType::RoomRedaction))
            }
            PowerLevelAction::SendMessage(msg_type) => self.for_message(msg_type),
            PowerLevelAction::SendState(state_type) => self.for_state(state_type),
            PowerLevelAction::TriggerNotification(NotificationPowerLevelType::Room) => {
                self.notifications.room
            }
        }
    }

    /// Get the power level required to send the given message type.
    pub fn for_message(&self, msg_type: MessageLikeEventType) -> Int {
        self.events.get(&msg_type.into()).copied().unwrap_or(self.events_default)
    }

    /// Get the power level required to send the given state event type.
    pub fn for_state(&self, state_type: StateEventType) -> Int {
        self.events.get(&state_type.into()).copied().unwrap_or(self.state_default)
    }

    /// Whether the given user can ban other users based on the power levels.
    ///
    /// Shorthand for `power_levels.user_can_do(user_id, PowerLevelAction::Ban)`.
    pub fn user_can_ban(&self, user_id: &UserId) -> bool {
        self.for_user(user_id) >= self.ban
    }

    /// Whether the acting user can ban the target user based on the power levels.
    ///
    /// On top of `power_levels.user_can_ban(acting_user_id)`, this performs an extra check
    /// to make sure the acting user has at greater power level than the target user.
    ///
    /// Shorthand for `power_levels.user_can_do_to_user(acting_user_id, target_user_id,
    /// PowerLevelUserAction::Ban)`.
    pub fn user_can_ban_user(&self, acting_user_id: &UserId, target_user_id: &UserId) -> bool {
        let acting_pl = self.for_user(acting_user_id);
        let target_pl = self.for_user(target_user_id);
        acting_pl >= self.ban && target_pl < acting_pl
    }

    /// Whether the given user can unban other users based on the power levels.
    ///
    /// This action requires to be allowed to ban and to kick.
    ///
    /// Shorthand for `power_levels.user_can_do(user_id, PowerLevelAction::Unban)`.
    pub fn user_can_unban(&self, user_id: &UserId) -> bool {
        let pl = self.for_user(user_id);
        pl >= self.ban && pl >= self.kick
    }

    /// Whether the acting user can unban the target user based on the power levels.
    ///
    /// This action requires to be allowed to ban and to kick.
    ///
    /// On top of `power_levels.user_can_unban(acting_user_id)`, this performs an extra check
    /// to make sure the acting user has at greater power level than the target user.
    ///
    /// Shorthand for `power_levels.user_can_do_to_user(acting_user_id, target_user_id,
    /// PowerLevelUserAction::Unban)`.
    pub fn user_can_unban_user(&self, acting_user_id: &UserId, target_user_id: &UserId) -> bool {
        let acting_pl = self.for_user(acting_user_id);
        let target_pl = self.for_user(target_user_id);
        acting_pl >= self.ban && acting_pl >= self.kick && target_pl < acting_pl
    }

    /// Whether the given user can invite other users based on the power levels.
    ///
    /// Shorthand for `power_levels.user_can_do(user_id, PowerLevelAction::Invite)`.
    pub fn user_can_invite(&self, user_id: &UserId) -> bool {
        self.for_user(user_id) >= self.invite
    }

    /// Whether the given user can kick other users based on the power levels.
    ///
    /// Shorthand for `power_levels.user_can_do(user_id, PowerLevelAction::Kick)`.
    pub fn user_can_kick(&self, user_id: &UserId) -> bool {
        self.for_user(user_id) >= self.kick
    }

    /// Whether the acting user can kick the target user based on the power levels.
    ///
    /// On top of `power_levels.user_can_kick(acting_user_id)`, this performs an extra check
    /// to make sure the acting user has at least the same power level as the target user.
    ///
    /// Shorthand for `power_levels.user_can_do_to_user(acting_user_id, target_user_id,
    /// PowerLevelUserAction::Kick)`.
    pub fn user_can_kick_user(&self, acting_user_id: &UserId, target_user_id: &UserId) -> bool {
        let acting_pl = self.for_user(acting_user_id);
        let target_pl = self.for_user(target_user_id);
        acting_pl >= self.kick && target_pl < acting_pl
    }

    /// Whether the given user can redact their own events based on the power levels.
    ///
    /// Shorthand for `power_levels.user_can_do(user_id, PowerLevelAction::RedactOwn)`.
    pub fn user_can_redact_own_event(&self, user_id: &UserId) -> bool {
        self.user_can_send_message(user_id, MessageLikeEventType::RoomRedaction)
    }

    /// Whether the given user can redact events of other users based on the power levels.
    ///
    /// Shorthand for `power_levels.user_can_do(user_id, PowerLevelAction::RedactOthers)`.
    pub fn user_can_redact_event_of_other(&self, user_id: &UserId) -> bool {
        self.user_can_redact_own_event(user_id) && self.for_user(user_id) >= self.redact
    }

    /// Whether the given user can send message events based on the power levels.
    ///
    /// Shorthand for `power_levels.user_can_do(user_id, PowerLevelAction::SendMessage(msg_type))`.
    pub fn user_can_send_message(&self, user_id: &UserId, msg_type: MessageLikeEventType) -> bool {
        self.for_user(user_id) >= self.for_message(msg_type)
    }

    /// Whether the given user can send state events based on the power levels.
    ///
    /// Shorthand for `power_levels.user_can_do(user_id, PowerLevelAction::SendState(state_type))`.
    pub fn user_can_send_state(&self, user_id: &UserId, state_type: StateEventType) -> bool {
        self.for_user(user_id) >= self.for_state(state_type)
    }

    /// Whether the given user can notify everybody in the room by writing `@room` in a message.
    ///
    /// Shorthand for `power_levels.user_can_do(user_id,
    /// PowerLevelAction::TriggerNotification(NotificationPowerLevelType::Room))`.
    pub fn user_can_trigger_room_notification(&self, user_id: &UserId) -> bool {
        self.for_user(user_id) >= self.notifications.room
    }

    /// Whether the acting user can change the power level of the target user.
    ///
    /// Shorthand for `power_levels.user_can_do_to_user(acting_user_id, target_user_id,
    /// PowerLevelUserAction::ChangePowerLevel`.
    pub fn user_can_change_user_power_level(
        &self,
        acting_user_id: &UserId,
        target_user_id: &UserId,
    ) -> bool {
        // Check that the user can change the power levels first.
        if !self.user_can_send_state(acting_user_id, StateEventType::RoomPowerLevels) {
            return false;
        }

        // A user can change their own power level.
        if acting_user_id == target_user_id {
            return true;
        }

        // The permission is different whether the target user is added or changed/removed, so
        // we need to check that.
        if let Some(target_pl) = self.users.get(target_user_id).copied() {
            self.for_user(acting_user_id) > target_pl
        } else {
            true
        }
    }

    /// Whether the given user can do the given action based on the power levels.
    pub fn user_can_do(&self, user_id: &UserId, action: PowerLevelAction) -> bool {
        match action {
            PowerLevelAction::Ban => self.user_can_ban(user_id),
            PowerLevelAction::Unban => self.user_can_unban(user_id),
            PowerLevelAction::Invite => self.user_can_invite(user_id),
            PowerLevelAction::Kick => self.user_can_kick(user_id),
            PowerLevelAction::RedactOwn => self.user_can_redact_own_event(user_id),
            PowerLevelAction::RedactOther => self.user_can_redact_event_of_other(user_id),
            PowerLevelAction::SendMessage(message_type) => {
                self.user_can_send_message(user_id, message_type)
            }
            PowerLevelAction::SendState(state_type) => {
                self.user_can_send_state(user_id, state_type)
            }
            PowerLevelAction::TriggerNotification(NotificationPowerLevelType::Room) => {
                self.user_can_trigger_room_notification(user_id)
            }
        }
    }

    /// Whether the acting user can do the given action to the target user based on the power
    /// levels.
    pub fn user_can_do_to_user(
        &self,
        acting_user_id: &UserId,
        target_user_id: &UserId,
        action: PowerLevelUserAction,
    ) -> bool {
        match action {
            PowerLevelUserAction::Ban => self.user_can_ban_user(acting_user_id, target_user_id),
            PowerLevelUserAction::Unban => self.user_can_unban_user(acting_user_id, target_user_id),
            PowerLevelUserAction::Invite => self.user_can_invite(acting_user_id),
            PowerLevelUserAction::Kick => self.user_can_kick_user(acting_user_id, target_user_id),
            PowerLevelUserAction::ChangePowerLevel => {
                self.user_can_change_user_power_level(acting_user_id, target_user_id)
            }
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
            invite: c.invite,
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

impl From<RoomPowerLevels> for PushConditionPowerLevelsCtx {
    fn from(c: RoomPowerLevels) -> Self {
        Self { users: c.users, users_default: c.users_default, notifications: c.notifications }
    }
}

/// The actions that can be limited by power levels.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PowerLevelAction {
    /// Ban a user.
    Ban,

    /// Unban a user.
    Unban,

    /// Invite a user.
    Invite,

    /// Kick a user.
    Kick,

    /// Redact one's own event.
    RedactOwn,

    /// Redact the event of another user.
    RedactOther,

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

/// The actions to other users that can be limited by power levels.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PowerLevelUserAction {
    /// Ban a user.
    Ban,

    /// Unban a user.
    Unban,

    /// Invite a user.
    Invite,

    /// Kick a user.
    Kick,

    /// Change a user's power level.
    ChangePowerLevel,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use assign::assign;
    use js_int::int;
    use maplit::btreemap;
    use ruma_common::user_id;
    use serde_json::{json, to_value as to_json_value};

    use super::{default_power_level, NotificationPowerLevels, RoomPowerLevelsEventContent};

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
