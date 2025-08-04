use std::collections::BTreeMap;

#[cfg(feature = "unstable-msc2654")]
use js_int::UInt;
use ruma_common::{serde::from_raw_json_value, OwnedEventId};
use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue as RawJsonValue;

use super::{
    Ephemeral, JoinedRoom, LeftRoom, RoomAccountData, RoomSummary, State, StateEvents, Timeline,
    UnreadNotificationsCount,
};

#[derive(Debug, Deserialize)]
struct StateDeHelper {
    state: Option<StateEvents>,
    #[cfg(feature = "unstable-msc4222")]
    #[serde(rename = "org.matrix.msc4222.state_after")]
    state_after: Option<StateEvents>,
}

impl<'de> Deserialize<'de> for State {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let StateDeHelper {
            state,
            #[cfg(feature = "unstable-msc4222")]
            state_after,
        } = StateDeHelper::deserialize(deserializer)?;

        #[cfg(feature = "unstable-msc4222")]
        if let Some(state) = state_after {
            return Ok(Self::After(state));
        }

        Ok(state.map(Self::Before).unwrap_or_default())
    }
}

#[derive(Debug, Deserialize)]
struct LeftRoomDeHelper {
    #[serde(default)]
    timeline: Timeline,
    #[serde(default)]
    account_data: RoomAccountData,
}

impl<'de> Deserialize<'de> for LeftRoom {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;

        let state = from_raw_json_value(&json)?;
        let LeftRoomDeHelper { timeline, account_data } = from_raw_json_value(&json)?;

        Ok(Self { timeline, state, account_data })
    }
}

#[derive(Debug, Deserialize)]
struct JoinedRoomDeHelper {
    #[serde(default)]
    summary: RoomSummary,
    #[serde(default)]
    unread_notifications: UnreadNotificationsCount,
    #[serde(default)]
    unread_thread_notifications: BTreeMap<OwnedEventId, UnreadNotificationsCount>,
    #[serde(default)]
    timeline: Timeline,
    #[serde(default)]
    account_data: RoomAccountData,
    #[serde(default)]
    ephemeral: Ephemeral,
    #[cfg(feature = "unstable-msc2654")]
    #[serde(rename = "org.matrix.msc2654.unread_count")]
    unread_count: Option<UInt>,
}

impl<'de> Deserialize<'de> for JoinedRoom {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;

        let state = from_raw_json_value(&json)?;
        let JoinedRoomDeHelper {
            summary,
            unread_notifications,
            unread_thread_notifications,
            timeline,
            account_data,
            ephemeral,
            #[cfg(feature = "unstable-msc2654")]
            unread_count,
        } = from_raw_json_value(&json)?;

        Ok(Self {
            summary,
            unread_notifications,
            unread_thread_notifications,
            timeline,
            state,
            account_data,
            ephemeral,
            #[cfg(feature = "unstable-msc2654")]
            unread_count,
        })
    }
}
