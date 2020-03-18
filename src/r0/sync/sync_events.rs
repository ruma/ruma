//! [GET /_matrix/client/r0/sync](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-sync)

use std::{collections::HashMap, time::Duration};

use js_int::UInt;
use ruma_api::{ruma_api, Outgoing};
use ruma_events::{
    collections::{
        all::{RoomEvent, StateEvent},
        only::Event as NonRoomEvent,
    },
    presence::PresenceEvent,
    stripped::AnyStrippedStateEvent,
    to_device::AnyToDeviceEvent,
    EventResult,
};
use ruma_identifiers::RoomId;
use serde::{Deserialize, Serialize};

use crate::r0::{
    filter::FilterDefinition,
    keys::KeyAlgorithm,
};

ruma_api! {
    metadata {
        description: "Get all new events from all rooms since the last sync or a given point of time.",
        method: GET,
        name: "sync",
        path: "/_matrix/client/r0/sync",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// A filter represented either as its full JSON definition or the ID of a saved filter.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub filter: Option<Filter>,
        /// A point in time to continue a sync from.
        /// Should be a token from the `next_batch` field of a previous `/sync`
        /// request.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub since: Option<String>,
        /// Controls whether to include the full state for all rooms the user is a member of.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub full_state: Option<bool>,
        /// Controls whether the client is automatically marked as online by polling this API.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub set_presence: Option<SetPresence>,
        /// The maximum time to poll in milliseconds before returning this request.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(with = "crate::serde::duration::opt_ms")]
        #[ruma_api(query)]
        pub timeout: Option<Duration>,
    }

    response {
        /// The batch token to supply in the `since` param of the next `/sync` request.
        pub next_batch: String,
        /// Updates to rooms.
        #[wrap_incoming]
        pub rooms: Rooms,
        /// Updates to the presence status of other users.
        #[wrap_incoming]
        pub presence: Presence,
        /// Messages sent dirrectly between devices.
        #[wrap_incoming]
        pub to_device: ToDevice,
        /// Information on E2E device updates.
        /// Only present on an incremental sync.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub device_lists: Option<DeviceLists>,
        /// For each key algorithm, the number of unclaimed one-time keys
        /// currently held on the server for a device.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub device_one_time_keys_count: Option<HashMap<KeyAlgorithm, UInt>>
    }

    error: crate::Error
}

/// Whether to set presence or not during sync.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SetPresence {
    /// Do not set the presence of the user calling this API.
    Offline,
    /// Mark client as online explicitly. Assumed by default.
    Online,
    /// Mark client as being idle.
    Unavailable,
}

/// A filter represented either as its full JSON definition or the ID of a saved filter.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::large_enum_variant)]
#[serde(untagged)]
pub enum Filter {
    // The filter definition needs to be (de)serialized twice because it is a URL-encoded JSON
    // string. Since #[ruma_api(query)] only does the latter and this is a very uncommon
    // setup, we implement it through custom serde logic for this specific enum variant rather than
    // adding another ruma_api attribute.
    //
    // On the deserialization side, because this is an enum with #[serde(untagged)], serde will
    // try the variants in order (https://serde.rs/enum-representations.html). That means because
    // FilterDefinition is the first variant, JSON decoding is attempted first which is almost
    // functionally equivalent to looking at whether the first symbol is a '{' as the spec says.
    // (there are probably some corner cases like leading whitespace)
    #[serde(with = "filter_def_serde")]
    /// A complete filter definition serialized to JSON.
    FilterDefinition(FilterDefinition),
    /// The ID of a filter saved on the server.
    FilterId(String),
}

/// Serialization and deserialization logic for filter definitions.
mod filter_def_serde {
    use serde::{de::Error as _, ser::Error as _, Deserialize, Deserializer, Serializer};

    use crate::r0::filter::FilterDefinition;

    /// Serialization logic for filter definitions.
    pub fn serialize<S>(filter_def: &FilterDefinition, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string = serde_json::to_string(filter_def).map_err(S::Error::custom)?;
        serializer.serialize_str(&string)
    }

    /// Deserialization logic for filter definitions.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<FilterDefinition, D::Error>
    where
        D: Deserializer<'de>,
    {
        let filter_str = <&str>::deserialize(deserializer)?;

        serde_json::from_str(filter_str).map_err(D::Error::custom)
    }
}

/// Updates to rooms.
#[derive(Clone, Debug, Serialize, Outgoing)]
pub struct Rooms {
    /// The rooms that the user has left or been banned from.
    #[wrap_incoming(LeftRoom)]
    pub leave: HashMap<RoomId, LeftRoom>,
    /// The rooms that the user has joined.
    #[wrap_incoming(JoinedRoom)]
    pub join: HashMap<RoomId, JoinedRoom>,
    /// The rooms that the user has been invited to.
    #[wrap_incoming(InvitedRoom)]
    pub invite: HashMap<RoomId, InvitedRoom>,
}

/// Historical updates to left rooms.
#[derive(Clone, Debug, Serialize, Outgoing)]
pub struct LeftRoom {
    /// The timeline of messages and state changes in the room up to the point when the user
    /// left.
    #[wrap_incoming]
    pub timeline: Timeline,
    /// The state updates for the room up to the start of the timeline.
    #[wrap_incoming]
    pub state: State,
    /// The private data that this user has attached to this room.
    #[wrap_incoming]
    pub account_data: AccountData,
}

/// Updates to joined rooms.
#[derive(Clone, Debug, Serialize, Outgoing)]
pub struct JoinedRoom {
    /// Information about the room which clients may need to correctly render it
    /// to users.
    pub summary: RoomSummary,
    /// Counts of unread notifications for this room.
    pub unread_notifications: UnreadNotificationsCount,
    /// The timeline of messages and state changes in the room.
    #[wrap_incoming]
    pub timeline: Timeline,
    /// Updates to the state, between the time indicated by the `since` parameter, and the start
    /// of the `timeline` (or all state up to the start of the `timeline`, if `since` is not
    /// given, or `full_state` is true).
    #[wrap_incoming]
    pub state: State,
    /// The private data that this user has attached to this room.
    #[wrap_incoming]
    pub account_data: AccountData,
    /// The ephemeral events in the room that aren't recorded in the timeline or state of the
    /// room. e.g. typing.
    #[wrap_incoming]
    pub ephemeral: Ephemeral,
}

/// unread notifications count
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct UnreadNotificationsCount {
    /// The number of unread notifications for this room with the highlight flag set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight_count: Option<UInt>,
    /// The total number of unread notifications for this room.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_count: Option<UInt>,
}

/// Events in the room.
#[derive(Clone, Debug, Serialize, Outgoing)]
pub struct Timeline {
    /// True if the number of events returned was limited by the `limit` on the filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limited: Option<bool>,
    /// A token that can be supplied to to the `from` parameter of the
    /// `/rooms/{roomId}/messages` endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_batch: Option<String>,
    /// A list of events.
    #[wrap_incoming(RoomEvent with EventResult)]
    pub events: Vec<RoomEvent>,
}

/// State events in the room.
#[derive(Clone, Debug, Serialize, Outgoing)]
pub struct State {
    /// A list of state events.
    #[wrap_incoming(StateEvent with EventResult)]
    pub events: Vec<StateEvent>,
}

/// The private data that this user has attached to this room.
#[derive(Clone, Debug, Serialize, Outgoing)]
pub struct AccountData {
    /// A list of events.
    // TODO: Create
    #[wrap_incoming(NonRoomEvent with EventResult)]
    pub events: Vec<NonRoomEvent>,
}

/// Ephemeral events not recorded in the timeline or state of the room.
#[derive(Clone, Debug, Serialize, Outgoing)]
pub struct Ephemeral {
    /// A list of events.
    #[wrap_incoming(NonRoomEvent with EventResult)]
    pub events: Vec<NonRoomEvent>,
}

/// Information about room for rendering to clients.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoomSummary {
    /// Users which can be used to generate a room name if the room does not have
    /// one. Required if room name or canonical aliases are not set or empty.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub heroes: Vec<String>,
    /// Number of users whose membership status is `join`.
    /// Required if field has changed since last sync; otherwise, it may be
    /// omitted.
    pub joined_member_count: Option<UInt>,
    /// Number of users whose membership status is `invite`.
    /// Required if field has changed since last sync; otherwise, it may be
    /// omitted.
    pub invited_member_count: Option<UInt>,
}

/// Updates to the rooms that the user has been invited to.
#[derive(Clone, Debug, Serialize, Outgoing)]
pub struct InvitedRoom {
    /// The state of a room that the user has been invited to.
    #[wrap_incoming]
    pub invite_state: InviteState,
}

/// The state of a room that the user has been invited to.
#[derive(Clone, Debug, Serialize, Outgoing)]
pub struct InviteState {
    /// A list of state events.
    #[wrap_incoming(AnyStrippedStateEvent with EventResult)]
    pub events: Vec<AnyStrippedStateEvent>,
}

/// Updates to the presence status of other users.
#[derive(Clone, Debug, Serialize, Outgoing)]
pub struct Presence {
    /// A list of events.
    #[wrap_incoming(PresenceEvent with EventResult)]
    pub events: Vec<PresenceEvent>,
}

/// Messages sent dirrectly between devices.
#[derive(Clone, Debug, Serialize, Outgoing)]
pub struct ToDevice {
    /// A list of to-device events.
    #[wrap_incoming(AnyToDeviceEvent with EventResult)]
    pub events: Vec<AnyToDeviceEvent>,
}

/// Information on E2E device udpates.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeviceLists {
    /// List of users who have updated their device identity keys or who now
    /// share an encrypted room with the client since the previous sync
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub changed: Vec<String>,
    /// List of users who no longer share encrypted rooms since the previous sync
    /// response.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub left: Vec<String>,

}