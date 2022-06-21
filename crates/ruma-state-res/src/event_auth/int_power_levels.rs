use std::collections::BTreeMap;

use js_int::Int;
use ruma_common::{
    events::{room::power_levels::RoomPowerLevelsEventContent, RoomEventType},
    power_levels::{default_power_level, NotificationPowerLevels},
    OwnedUserId,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct IntRoomPowerLevelsEventContent {
    #[serde(default = "default_power_level")]
    pub ban: Int,

    #[serde(default)]
    pub events: BTreeMap<RoomEventType, Int>,

    #[serde(default)]
    pub events_default: Int,

    #[serde(default)]
    pub invite: Int,

    #[serde(default = "default_power_level")]
    pub kick: Int,

    #[serde(default = "default_power_level")]
    pub redact: Int,

    #[serde(default = "default_power_level")]
    pub state_default: Int,

    #[serde(default)]
    pub users: BTreeMap<OwnedUserId, Int>,

    #[serde(default)]
    pub users_default: Int,

    #[serde(default)]
    pub notifications: IntNotificationPowerLevels,
}

impl From<IntRoomPowerLevelsEventContent> for RoomPowerLevelsEventContent {
    fn from(int_pl: IntRoomPowerLevelsEventContent) -> Self {
        let IntRoomPowerLevelsEventContent {
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
        } = int_pl;

        let mut pl = Self::new();
        pl.ban = ban;
        pl.events = events;
        pl.events_default = events_default;
        pl.invite = invite;
        pl.kick = kick;
        pl.redact = redact;
        pl.state_default = state_default;
        pl.users = users;
        pl.users_default = users_default;
        pl.notifications = notifications.into();

        pl
    }
}

#[derive(Deserialize)]
pub struct IntNotificationPowerLevels {
    #[serde(default = "default_power_level")]
    pub room: Int,
}

impl Default for IntNotificationPowerLevels {
    fn default() -> Self {
        Self { room: default_power_level() }
    }
}

impl From<IntNotificationPowerLevels> for NotificationPowerLevels {
    fn from(int_notif: IntNotificationPowerLevels) -> Self {
        let mut notif = Self::new();
        notif.room = int_notif.room;

        notif
    }
}
