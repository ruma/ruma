//! Types to deserialize `m.room.power_levels` events.

use std::{
    collections::{BTreeMap, HashSet},
    ops::Deref,
    sync::{Arc, Mutex, OnceLock},
};

use js_int::{Int, int};
use ruma_common::{
    UserId,
    room_version_rules::AuthorizationRules,
    serde::{
        DebugAsRefStr, DisplayAsRefStr, EqAsRefStr, JsonObject, OrdAsRefStr,
        btreemap_deserialize_v1_powerlevel_values, deserialize_v1_powerlevel, from_raw_json_value,
    },
};
use ruma_events::{TimelineEventType, room::power_levels::UserPowerLevel};
use serde::de::DeserializeOwned;
use serde_json::{Error, from_value as from_json_value};

use super::Event;

/// The default value of the creator's power level.
const DEFAULT_CREATOR_POWER_LEVEL: i32 = 100;

/// A helper type for an [`Event`] of type `m.room.power_levels`.
///
/// This is a type that deserializes each field lazily, when requested. Some deserialization results
/// are cached in memory, if they are used often.
#[derive(Debug, Clone)]
pub struct RoomPowerLevelsEvent<E: Event> {
    inner: Arc<RoomPowerLevelsEventInner<E>>,
}

#[derive(Debug)]
struct RoomPowerLevelsEventInner<E: Event> {
    /// The inner `Event`.
    event: E,

    /// The deserialized content of the event.
    deserialized_content: OnceLock<JsonObject>,

    /// The values of fields that should contain an integer.
    int_fields: Mutex<BTreeMap<RoomPowerLevelsIntField, Option<Int>>>,

    /// The power levels of the users, if any.
    users: OnceLock<Option<BTreeMap<UserId, Int>>>,
}

impl<E: Event> RoomPowerLevelsEvent<E> {
    /// Construct a new `RoomPowerLevelsEvent` around the given event.
    pub fn new(event: E) -> Self {
        Self {
            inner: RoomPowerLevelsEventInner {
                event,
                deserialized_content: OnceLock::new(),
                int_fields: Mutex::new(BTreeMap::new()),
                users: OnceLock::new(),
            }
            .into(),
        }
    }

    /// The deserialized content of the event.
    fn deserialized_content(&self) -> Result<&JsonObject, String> {
        // TODO: Use OnceLock::get_or_try_init when it is stabilized.
        if let Some(content) = self.inner.deserialized_content.get() {
            Ok(content)
        } else {
            let content = from_raw_json_value(self.content()).map_err(|error: Error| {
                format!("malformed `m.room.power_levels` content: {error}")
            })?;
            Ok(self.inner.deserialized_content.get_or_init(|| content))
        }
    }

    /// Get the value of a field that should contain an integer, if any.
    ///
    /// The deserialization of this field is cached in memory.
    pub fn get_as_int(
        &self,
        field: RoomPowerLevelsIntField,
        rules: &AuthorizationRules,
    ) -> Result<Option<Int>, String> {
        let mut int_fields =
            self.inner.int_fields.lock().expect("we never panic while holding the mutex");

        if let Some(power_level) = int_fields.get(&field) {
            return Ok(*power_level);
        }

        let content = self.deserialized_content()?;

        let Some(value) = content.get(field.as_str()) else {
            int_fields.insert(field, None);
            return Ok(None);
        };

        let res = if rules.integer_power_levels {
            from_json_value(value.clone())
        } else {
            deserialize_v1_powerlevel(value)
        };

        let power_level = res.map(Some).map_err(|error| {
            format!(
                "unexpected format of `{field}` field in `content` \
                 of `m.room.power_levels` event: {error}"
            )
        })?;

        int_fields.insert(field, power_level);
        Ok(power_level)
    }

    /// Get the value of a field that should contain an integer, or its default value if it is
    /// absent.
    pub fn get_as_int_or_default(
        &self,
        field: RoomPowerLevelsIntField,
        rules: &AuthorizationRules,
    ) -> Result<Int, String> {
        Ok(self.get_as_int(field, rules)?.unwrap_or_else(|| field.default_value()))
    }

    /// Get the value of a field that should contain a map of any value to integer, if any.
    fn get_as_int_map<T: Ord + DeserializeOwned>(
        &self,
        field: &str,
        rules: &AuthorizationRules,
    ) -> Result<Option<BTreeMap<T, Int>>, String> {
        let content = self.deserialized_content()?;

        let Some(value) = content.get(field) else {
            return Ok(None);
        };

        let res = if rules.integer_power_levels {
            from_json_value(value.clone())
        } else {
            btreemap_deserialize_v1_powerlevel_values(value)
        };

        res.map(Some).map_err(|error| {
            format!(
                "unexpected format of `{field}` field in `content` \
                 of `m.room.power_levels` event: {error}"
            )
        })
    }

    /// Get the power levels required to send events, if any.
    pub fn events(
        &self,
        rules: &AuthorizationRules,
    ) -> Result<Option<BTreeMap<TimelineEventType, Int>>, String> {
        self.get_as_int_map("events", rules)
    }

    /// Get the power levels required to trigger notifications, if any.
    pub fn notifications(
        &self,
        rules: &AuthorizationRules,
    ) -> Result<Option<BTreeMap<String, Int>>, String> {
        self.get_as_int_map("notifications", rules)
    }

    /// Get the power levels of the users, if any.
    ///
    /// The deserialization of this field is cached in memory.
    pub fn users(
        &self,
        rules: &AuthorizationRules,
    ) -> Result<Option<&BTreeMap<UserId, Int>>, String> {
        // TODO: Use OnceLock::get_or_try_init when it is stabilized.
        if let Some(users) = self.inner.users.get() {
            Ok(users.as_ref())
        } else {
            let users = self.get_as_int_map("users", rules)?;
            Ok(self.inner.users.get_or_init(|| users).as_ref())
        }
    }

    /// Get the power level of the user with the given ID.
    ///
    /// Calling this method several times should be cheap because the necessary deserialization
    /// results are cached.
    pub fn user_power_level(
        &self,
        user_id: &UserId,
        rules: &AuthorizationRules,
    ) -> Result<Int, String> {
        if let Some(power_level) = self.users(rules)?.as_ref().and_then(|users| users.get(user_id))
        {
            return Ok(*power_level);
        }

        self.get_as_int_or_default(RoomPowerLevelsIntField::UsersDefault, rules)
    }

    /// Get the power level required to send an event of the given type.
    pub fn event_power_level(
        &self,
        event_type: &TimelineEventType,
        state_key: Option<&str>,
        rules: &AuthorizationRules,
    ) -> Result<Int, String> {
        let events = self.events(rules)?;

        if let Some(power_level) = events.as_ref().and_then(|events| events.get(event_type)) {
            return Ok(*power_level);
        }

        let default_field = if state_key.is_some() {
            RoomPowerLevelsIntField::StateDefault
        } else {
            RoomPowerLevelsIntField::EventsDefault
        };

        self.get_as_int_or_default(default_field, rules)
    }

    /// Get a map of all the fields with an integer value in the `content` of an
    /// `m.room.power_levels` event.
    pub(crate) fn int_fields_map(
        &self,
        rules: &AuthorizationRules,
    ) -> Result<BTreeMap<RoomPowerLevelsIntField, Int>, String> {
        RoomPowerLevelsIntField::ALL
            .iter()
            .copied()
            .filter_map(|field| match self.get_as_int(field, rules) {
                Ok(value) => value.map(|value| Ok((field, value))),
                Err(error) => Some(Err(error)),
            })
            .collect()
    }
}

impl<E: Event> Deref for RoomPowerLevelsEvent<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.inner.event
    }
}

/// Helper trait for `Option<RoomPowerLevelsEvent<E>>`.
pub(crate) trait RoomPowerLevelsEventOptionExt {
    /// Get the power level of the user with the given ID.
    fn user_power_level(
        &self,
        user_id: &UserId,
        creators: &HashSet<UserId>,
        rules: &AuthorizationRules,
    ) -> Result<UserPowerLevel, String>;

    /// Get the value of a field that should contain an integer, or its default value if it is
    /// absent.
    fn get_as_int_or_default(
        &self,
        field: RoomPowerLevelsIntField,
        rules: &AuthorizationRules,
    ) -> Result<Int, String>;

    /// Get the power level required to send an event of the given type.
    fn event_power_level(
        &self,
        event_type: &TimelineEventType,
        state_key: Option<&str>,
        rules: &AuthorizationRules,
    ) -> Result<Int, String>;
}

impl<E: Event> RoomPowerLevelsEventOptionExt for Option<RoomPowerLevelsEvent<E>> {
    fn user_power_level(
        &self,
        user_id: &UserId,
        creators: &HashSet<UserId>,
        rules: &AuthorizationRules,
    ) -> Result<UserPowerLevel, String> {
        if rules.explicitly_privilege_room_creators && creators.contains(user_id) {
            Ok(UserPowerLevel::Infinite)
        } else if let Some(room_power_levels_event) = self {
            room_power_levels_event.user_power_level(user_id, rules).map(Into::into)
        } else {
            let power_level = if creators.contains(user_id) {
                DEFAULT_CREATOR_POWER_LEVEL.into()
            } else {
                RoomPowerLevelsIntField::UsersDefault.default_value()
            };
            Ok(power_level.into())
        }
    }

    fn get_as_int_or_default(
        &self,
        field: RoomPowerLevelsIntField,
        rules: &AuthorizationRules,
    ) -> Result<Int, String> {
        if let Some(room_power_levels_event) = self {
            room_power_levels_event.get_as_int_or_default(field, rules)
        } else {
            Ok(field.default_value())
        }
    }

    fn event_power_level(
        &self,
        event_type: &TimelineEventType,
        state_key: Option<&str>,
        rules: &AuthorizationRules,
    ) -> Result<Int, String> {
        if let Some(room_power_levels_event) = self {
            room_power_levels_event.event_power_level(event_type, state_key, rules)
        } else {
            let default_field = if state_key.is_some() {
                RoomPowerLevelsIntField::StateDefault
            } else {
                RoomPowerLevelsIntField::EventsDefault
            };

            Ok(default_field.default_value())
        }
    }
}

/// Fields in the `content` of an `m.room.power_levels` event with an integer value.
#[derive(DebugAsRefStr, Clone, Copy, DisplayAsRefStr, EqAsRefStr, OrdAsRefStr)]
#[non_exhaustive]
pub enum RoomPowerLevelsIntField {
    /// `users_default`
    UsersDefault,

    /// `events_default`
    EventsDefault,

    /// `state_default`
    StateDefault,

    /// `ban`
    Ban,

    /// `redact`
    Redact,

    /// `kick`
    Kick,

    /// `invite`
    Invite,
}

impl RoomPowerLevelsIntField {
    /// A slice containing all the variants.
    pub(crate) const ALL: &[RoomPowerLevelsIntField] = &[
        Self::UsersDefault,
        Self::EventsDefault,
        Self::StateDefault,
        Self::Ban,
        Self::Redact,
        Self::Kick,
        Self::Invite,
    ];

    /// The string representation of this field.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }

    /// The default value for this field if it is absent.
    pub fn default_value(&self) -> Int {
        match self {
            Self::UsersDefault | Self::EventsDefault | Self::Invite => int!(0),
            Self::StateDefault | Self::Kick | Self::Ban | Self::Redact => int!(50),
        }
    }
}

impl AsRef<str> for RoomPowerLevelsIntField {
    fn as_ref(&self) -> &str {
        match self {
            Self::UsersDefault => "users_default",
            Self::EventsDefault => "events_default",
            Self::StateDefault => "state_default",
            Self::Ban => "ban",
            Self::Redact => "redact",
            Self::Kick => "kick",
            Self::Invite => "invite",
        }
    }
}
