//! Types for the [`m.room.power_levels`] event.
//!
//! [`m.room.power_levels`]: https://spec.matrix.org/latest/client-server-api/#mroompower_levels

use std::{
    cmp::{Ordering, max},
    collections::BTreeMap,
};

use js_int::{Int, int};
use ruma_common::{
    OwnedUserId, UserId,
    power_levels::{NotificationPowerLevels, default_power_level},
    push::PushConditionPowerLevelsCtx,
    room_version_rules::{AuthorizationRules, RedactionRules, RoomPowerLevelsRules},
};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{
    EmptyStateKey, MessageLikeEventType, RedactContent, RedactedStateEventContent, StateEventType,
    StaticEventContent, TimelineEventType,
};

/// The content of an `m.room.power_levels` event.
///
/// Defines the power levels (privileges) of users in the room.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
    /// Creates a new `RoomPowerLevelsEventContent` with all-default values for the given
    /// authorization rules.
    pub fn new(rules: &AuthorizationRules) -> Self {
        // events_default, users_default and invite having a default of 0 while the others have a
        // default of 50 is not an oversight, these defaults are from the Matrix specification.
        let mut pl = Self {
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
        };

        if rules.explicitly_privilege_room_creators {
            // Since v12, the default power level to send m.room.tombstone events is increased to
            // PL150.
            pl.events.insert(TimelineEventType::RoomTombstone, int!(150));
        }

        pl
    }
}

impl RedactContent for RoomPowerLevelsEventContent {
    type Redacted = RedactedRoomPowerLevelsEventContent;

    fn redact(self, rules: &RedactionRules) -> Self::Redacted {
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

        let invite = if rules.keep_room_power_levels_invite { invite } else { int!(0) };

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
    pub fn power_levels(
        &self,
        rules: &AuthorizationRules,
        creators: Vec<OwnedUserId>,
    ) -> RoomPowerLevels {
        match self {
            Self::Original(ev) => RoomPowerLevels::new(ev.content.clone().into(), rules, creators),
            Self::Redacted(ev) => RoomPowerLevels::new(ev.content.clone().into(), rules, creators),
        }
    }
}

impl SyncRoomPowerLevelsEvent {
    /// Obtain the effective power levels, regardless of whether this event is redacted.
    pub fn power_levels(
        &self,
        rules: &AuthorizationRules,
        creators: Vec<OwnedUserId>,
    ) -> RoomPowerLevels {
        match self {
            Self::Original(ev) => RoomPowerLevels::new(ev.content.clone().into(), rules, creators),
            Self::Redacted(ev) => RoomPowerLevels::new(ev.content.clone().into(), rules, creators),
        }
    }
}

impl StrippedRoomPowerLevelsEvent {
    /// Obtain the effective power levels from this event.
    pub fn power_levels(
        &self,
        rules: &AuthorizationRules,
        creators: Vec<OwnedUserId>,
    ) -> RoomPowerLevels {
        RoomPowerLevels::new(self.content.clone().into(), rules, creators)
    }
}

/// Redacted form of [`RoomPowerLevelsEventContent`].
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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

impl StaticEventContent for RedactedRoomPowerLevelsEventContent {
    const TYPE: &'static str = RoomPowerLevelsEventContent::TYPE;
    type IsPrefix = <RoomPowerLevelsEventContent as StaticEventContent>::IsPrefix;
}

impl RedactedStateEventContent for RedactedRoomPowerLevelsEventContent {
    type StateKey = EmptyStateKey;

    fn event_type(&self) -> StateEventType {
        StateEventType::RoomPowerLevels
    }
}

impl From<RedactedRoomPowerLevelsEventContent> for PossiblyRedactedRoomPowerLevelsEventContent {
    fn from(value: RedactedRoomPowerLevelsEventContent) -> Self {
        let RedactedRoomPowerLevelsEventContent {
            ban,
            events,
            events_default,
            invite,
            kick,
            redact,
            state_default,
            users,
            users_default,
        } = value;

        Self {
            ban,
            events,
            events_default,
            invite,
            kick,
            redact,
            state_default,
            users,
            users_default,
            notifications: NotificationPowerLevels::default(),
        }
    }
}

/// The power level of a particular user.
///
/// Is either considered "infinite" if that user is a room creator, or an integer if they are not.
#[derive(PartialEq, Copy, Clone, Eq, Debug)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum UserPowerLevel {
    /// The user is considered to have "infinite" power level, due to being a room creator, from
    /// room version 12 onwards.
    Infinite,

    /// The user is either not a creator, or the room version is prior to 12, and hence has an
    /// integer power level.
    Int(Int),
}

impl Ord for UserPowerLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (UserPowerLevel::Infinite, UserPowerLevel::Infinite) => Ordering::Equal,
            (UserPowerLevel::Infinite, UserPowerLevel::Int(_)) => Ordering::Greater,
            (UserPowerLevel::Int(_), UserPowerLevel::Infinite) => Ordering::Less,
            (UserPowerLevel::Int(self_int), UserPowerLevel::Int(other_int)) => {
                self_int.cmp(other_int)
            }
        }
    }
}

impl PartialOrd for UserPowerLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq<Int> for UserPowerLevel {
    fn eq(&self, other: &Int) -> bool {
        match self {
            UserPowerLevel::Infinite => false,
            UserPowerLevel::Int(int) => int.eq(other),
        }
    }
}

impl PartialEq<UserPowerLevel> for Int {
    fn eq(&self, other: &UserPowerLevel) -> bool {
        other.eq(self)
    }
}

impl PartialOrd<Int> for UserPowerLevel {
    fn partial_cmp(&self, other: &Int) -> Option<Ordering> {
        match self {
            UserPowerLevel::Infinite => Some(Ordering::Greater),
            UserPowerLevel::Int(int) => int.partial_cmp(other),
        }
    }
}

impl PartialOrd<UserPowerLevel> for Int {
    fn partial_cmp(&self, other: &UserPowerLevel) -> Option<Ordering> {
        match other {
            UserPowerLevel::Infinite => Some(Ordering::Less),
            UserPowerLevel::Int(int) => self.partial_cmp(int),
        }
    }
}

impl From<Int> for UserPowerLevel {
    fn from(value: Int) -> Self {
        Self::Int(value)
    }
}

/// The effective power levels of a room.
///
/// This struct contains all the power levels settings from the specification and can be constructed
/// from several [`RoomPowerLevelsSource`]s, which means that it can be used when wanting to inspect
/// the power levels of a room, regardless of whether the most recent power levels event is redacted
/// or not, or the room has no power levels event.
///
/// This can also be used to change the power levels of a room by mutating it and then converting it
/// to a [`RoomPowerLevelsEventContent`] using `RoomPowerLevelsEventContent::try_from` /
/// `.try_into()`. This allows to validate the format of the power levels before sending them. Note
/// that the homeserver might still refuse the power levels changes depending on the current power
/// level of the sender.
#[derive(Clone, Debug)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RoomPowerLevels {
    /// The level required to ban a user.
    ///
    /// When built from [`RoomPowerLevelsSource::None`], defaults to `50`.
    pub ban: Int,

    /// The level required to send specific event types.
    ///
    /// This is a mapping from event type to power level required.
    ///
    /// When built from [`RoomPowerLevelsSource::None`], defaults to an empty map.
    pub events: BTreeMap<TimelineEventType, Int>,

    /// The default level required to send message events.
    ///
    /// When built from [`RoomPowerLevelsSource::None`], defaults to `0`.
    pub events_default: Int,

    /// The level required to invite a user.
    ///
    /// When built from [`RoomPowerLevelsSource::None`], defaults to `0`.
    pub invite: Int,

    /// The level required to kick a user.
    ///
    /// When built from [`RoomPowerLevelsSource::None`], defaults to `50`.
    pub kick: Int,

    /// The level required to redact an event.
    ///
    /// When built from [`RoomPowerLevelsSource::None`], defaults to `50`.
    pub redact: Int,

    /// The default level required to send state events.
    ///
    /// When built from [`RoomPowerLevelsSource::None`], defaults to `50`.
    pub state_default: Int,

    /// The power levels for specific users.
    ///
    /// This is a mapping from `user_id` to power level for that user.
    ///
    /// Must NOT contain creators of the room in room versions where the
    /// `explicitly_privilege_room_creators` field of [`AuthorizationRules`] is set to `true`. This
    /// would result in an error when trying to convert this to a [`RoomPowerLevelsEventContent`].
    ///
    /// When built from [`RoomPowerLevelsSource::None`]:
    ///
    /// * If `explicitly_privilege_room_creators` is set to `false` for the room version, defaults
    ///   to setting the power level to `100` for the creator(s) of the room.
    /// * Otherwise, defaults to an empty map.
    pub users: BTreeMap<OwnedUserId, Int>,

    /// The default power level for every user in the room.
    ///
    /// When built from [`RoomPowerLevelsSource::None`], defaults to `0`.
    pub users_default: Int,

    /// The power level requirements for specific notification types.
    ///
    /// This is a mapping from `key` to power level for that notifications key.
    ///
    /// When built from [`RoomPowerLevelsSource::None`], uses its `Default` implementation.
    pub notifications: NotificationPowerLevels,

    /// The tweaks for determining the power level of a user.
    pub rules: RoomPowerLevelsRules,
}

impl RoomPowerLevels {
    /// Constructs `RoomPowerLevels` from `RoomPowerLevelsSource`, `AuthorizationRules` and the
    /// creators of a room.
    pub fn new(
        power_levels: RoomPowerLevelsSource,
        rules: &AuthorizationRules,
        creators: impl IntoIterator<Item = OwnedUserId> + Clone,
    ) -> Self {
        match power_levels {
            RoomPowerLevelsSource::Original(RoomPowerLevelsEventContent {
                ban,
                events,
                events_default,
                invite,
                kick,
                redact,
                state_default,
                users,
                users_default,
                notifications,
            }) => Self {
                ban,
                events,
                events_default,
                invite,
                kick,
                redact,
                state_default,
                users,
                users_default,
                notifications,
                rules: RoomPowerLevelsRules::new(rules, creators),
            },
            RoomPowerLevelsSource::Redacted(RedactedRoomPowerLevelsEventContent {
                ban,
                events,
                events_default,
                invite,
                kick,
                redact,
                state_default,
                users,
                users_default,
            }) => Self {
                ban,
                events,
                events_default,
                invite,
                kick,
                redact,
                state_default,
                users,
                users_default,
                notifications: NotificationPowerLevels::new(),
                rules: RoomPowerLevelsRules::new(rules, creators),
            },
            // events_default, users_default and invite having a default of 0 while the others have
            // a default of 50 is not an oversight, these defaults are from the Matrix
            // specification.
            RoomPowerLevelsSource::None => Self {
                ban: default_power_level(),
                events: BTreeMap::new(),
                events_default: int!(0),
                invite: int!(0),
                kick: default_power_level(),
                redact: default_power_level(),
                state_default: default_power_level(),
                users: if rules.explicitly_privilege_room_creators {
                    BTreeMap::new()
                } else {
                    // If creators are not explicitly privileged, their power level is 100 if there
                    // is no power levels state.
                    BTreeMap::from_iter(creators.clone().into_iter().map(|user| (user, int!(100))))
                },
                users_default: int!(0),
                notifications: NotificationPowerLevels::default(),
                rules: RoomPowerLevelsRules::new(rules, creators),
            },
        }
    }

    /// Whether the given user ID is a privileged creator.
    fn is_privileged_creator(&self, user_id: &UserId) -> bool {
        self.rules.privileged_creators.as_ref().is_some_and(|creators| creators.contains(user_id))
    }

    /// Get the power level of a specific user.
    pub fn for_user(&self, user_id: &UserId) -> UserPowerLevel {
        if self.is_privileged_creator(user_id) {
            return UserPowerLevel::Infinite;
        }

        self.users.get(user_id).map_or(self.users_default, |pl| *pl).into()
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

        // No one can change the power level of a privileged creator.
        if self.is_privileged_creator(target_user_id) {
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

/// An error encountered when trying to build a [`RoomPowerLevelsEventContent`] from
/// [`RoomPowerLevels`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum PowerLevelsError {
    /// A creator is in the `users` map, and it is not allowed by the current room version.
    #[error("a creator is in the `users` map, and it is not allowed by the current room version")]
    CreatorInUsersMap,
}

impl TryFrom<RoomPowerLevels> for RoomPowerLevelsEventContent {
    type Error = PowerLevelsError;

    fn try_from(c: RoomPowerLevels) -> Result<Self, Self::Error> {
        if c.rules.privileged_creators.as_ref().is_some_and(|creators| {
            !c.users.is_empty() && creators.iter().any(|user_id| c.users.contains_key(user_id))
        }) {
            return Err(PowerLevelsError::CreatorInUsersMap);
        }

        Ok(Self {
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
        })
    }
}

impl From<RoomPowerLevels> for PushConditionPowerLevelsCtx {
    fn from(c: RoomPowerLevels) -> Self {
        Self::new(c.users, c.users_default, c.notifications, c.rules)
    }
}

/// The possible power level sources for [`RoomPowerLevels`].
#[derive(Default)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum RoomPowerLevelsSource {
    /// Construct `RoomPowerLevels` from the non-redacted `m.room.power_levels` event content.
    Original(RoomPowerLevelsEventContent),
    /// Construct `RoomPowerLevels` from the redacted `m.room.power_levels` event content.
    Redacted(RedactedRoomPowerLevelsEventContent),
    /// Use the default values defined in the specification.
    ///
    /// Should only be used when there is no power levels state in a room.
    #[default]
    None,
}

impl From<Option<RoomPowerLevelsEventContent>> for RoomPowerLevelsSource {
    fn from(value: Option<RoomPowerLevelsEventContent>) -> Self {
        value.map(Self::Original).unwrap_or_default()
    }
}

impl From<Option<RedactedRoomPowerLevelsEventContent>> for RoomPowerLevelsSource {
    fn from(value: Option<RedactedRoomPowerLevelsEventContent>) -> Self {
        value.map(Self::Redacted).unwrap_or_default()
    }
}

impl From<RoomPowerLevelsEventContent> for RoomPowerLevelsSource {
    fn from(value: RoomPowerLevelsEventContent) -> Self {
        Self::Original(value)
    }
}

impl From<RedactedRoomPowerLevelsEventContent> for RoomPowerLevelsSource {
    fn from(value: RedactedRoomPowerLevelsEventContent) -> Self {
        Self::Redacted(value)
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
    use ruma_common::{
        canonical_json::assert_to_canonical_json_eq, owned_user_id,
        room_version_rules::AuthorizationRules, user_id,
    };
    use serde_json::json;

    use super::{
        NotificationPowerLevels, RoomPowerLevels, RoomPowerLevelsEventContent,
        RoomPowerLevelsSource, default_power_level,
    };

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

        assert_to_canonical_json_eq!(power_levels, json!({}));
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

        assert_to_canonical_json_eq!(
            power_levels_event,
            json!({
                "ban": 23,
                "events": {
                    "m.dummy": 23,
                },
                "events_default": 23,
                "invite": 23,
                "kick": 23,
                "redact": 23,
                "state_default": 23,
                "users": {
                    "@carl:example.com": 23,
                },
                "users_default": 23,
                "notifications": {
                    "room": 23,
                },
            }),
        );
    }

    #[test]
    fn cannot_change_power_level_of_privileged_creator() {
        let creator = user_id!("@lola:localhost");

        let v1_power_levels = RoomPowerLevels::new(
            RoomPowerLevelsSource::None,
            &AuthorizationRules::V1,
            vec![creator.to_owned()],
        );
        assert!(v1_power_levels.user_can_change_user_power_level(creator, creator));

        let v12_power_levels = RoomPowerLevels::new(
            RoomPowerLevelsSource::None,
            &AuthorizationRules::V12,
            vec![creator.to_owned()],
        );
        assert!(!v12_power_levels.user_can_change_user_power_level(creator, creator));
    }

    #[test]
    fn cannot_convert_to_event_content_with_creator_in_users() {
        let creator = owned_user_id!("@lola:localhost");

        let mut v1_power_levels = RoomPowerLevels::new(
            RoomPowerLevelsSource::None,
            &AuthorizationRules::V1,
            vec![creator.clone()],
        );
        v1_power_levels.users.insert(creator.clone(), int!(75));
        let v1_event_content = RoomPowerLevelsEventContent::try_from(v1_power_levels).unwrap();
        assert_eq!(*v1_event_content.users.get(&creator).unwrap(), int!(75));

        let mut v12_power_levels = RoomPowerLevels::new(
            RoomPowerLevelsSource::None,
            &AuthorizationRules::V12,
            vec![creator.to_owned()],
        );
        v12_power_levels.users.insert(creator.clone(), int!(75));
        RoomPowerLevelsEventContent::try_from(v12_power_levels).unwrap_err();
    }
}
