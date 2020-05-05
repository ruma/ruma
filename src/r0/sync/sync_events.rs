//! [GET /_matrix/client/r0/sync](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-sync)

use std::{collections::BTreeMap, time::Duration};

use js_int::UInt;
use ruma_api::ruma_api;
use ruma_events::{
    collections::{
        all::{RoomEvent, StateEvent},
        only::Event as NonRoomEvent,
    },
    presence::PresenceEvent,
    stripped::AnyStrippedStateEvent,
    to_device::AnyToDeviceEvent,
    EventJson,
};
use ruma_identifiers::RoomId;
use serde::{Deserialize, Serialize};

use crate::r0::{filter::FilterDefinition, keys::KeyAlgorithm};

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
        ///
        /// Should be a token from the `next_batch` field of a previous `/sync`
        /// request.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub since: Option<String>,

        /// Controls whether to include the full state for all rooms the user is a member of.
        #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
        #[ruma_api(query)]
        pub full_state: bool,

        /// Controls whether the client is automatically marked as online by polling this API.
        #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
        #[ruma_api(query)]
        pub set_presence: SetPresence,

        /// The maximum time to poll in milliseconds before returning this request.
        #[serde(
            with = "ruma_serde::duration::opt_ms",
            default,
            skip_serializing_if = "Option::is_none",
        )]
        #[ruma_api(query)]
        pub timeout: Option<Duration>,
    }

    response {
        /// The batch token to supply in the `since` param of the next `/sync` request.
        pub next_batch: String,

        /// Updates to rooms.
        #[serde(default, skip_serializing_if = "Rooms::is_empty")]
        pub rooms: Rooms,

        /// Updates to the presence status of other users.
        #[serde(default, skip_serializing_if = "Presence::is_empty")]
        pub presence: Presence,

        /// The global private data created by this user.
        #[serde(default, skip_serializing_if = "AccountData::is_empty")]
        pub account_data: AccountData,

        /// Messages sent dirrectly between devices.
        #[serde(default, skip_serializing_if = "ToDevice::is_empty")]
        pub to_device: ToDevice,

        /// Information on E2E device updates.
        ///
        /// Only present on an incremental sync.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub device_lists: Option<DeviceLists>,

        /// For each key algorithm, the number of unclaimed one-time keys
        /// currently held on the server for a device.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub device_one_time_keys_count: BTreeMap<KeyAlgorithm, UInt>,
    }

    error: crate::Error
}

/// Whether to set presence or not during sync.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SetPresence {
    /// Do not set the presence of the user calling this API.
    Offline,

    /// Mark client as online explicitly. Assumed by default.
    Online,

    /// Mark client as being idle.
    Unavailable,
}

impl Default for SetPresence {
    fn default() -> Self {
        Self::Online
    }
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
    #[serde(with = "ruma_serde::json_string")]
    /// A complete filter definition serialized to JSON.
    FilterDefinition(FilterDefinition),

    /// The ID of a filter saved on the server.
    FilterId(String),
}

/// Updates to rooms.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Rooms {
    /// The rooms that the user has left or been banned from.
    pub leave: BTreeMap<RoomId, LeftRoom>,

    /// The rooms that the user has joined.
    pub join: BTreeMap<RoomId, JoinedRoom>,

    /// The rooms that the user has been invited to.
    pub invite: BTreeMap<RoomId, InvitedRoom>,
}

impl Rooms {
    fn is_empty(&self) -> bool {
        self.leave.is_empty() && self.join.is_empty() && self.invite.is_empty()
    }
}

/// Historical updates to left rooms.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeftRoom {
    /// The timeline of messages and state changes in the room up to the point when the user
    /// left.
    pub timeline: Timeline,

    /// The state updates for the room up to the start of the timeline.
    pub state: State,

    /// The private data that this user has attached to this room.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_data: Option<AccountData>,
}

/// Updates to joined rooms.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JoinedRoom {
    /// Information about the room which clients may need to correctly render it
    /// to users.
    pub summary: RoomSummary,

    /// Counts of unread notifications for this room.
    pub unread_notifications: UnreadNotificationsCount,

    /// The timeline of messages and state changes in the room.
    pub timeline: Timeline,

    /// Updates to the state, between the time indicated by the `since` parameter, and the start
    /// of the `timeline` (or all state up to the start of the `timeline`, if `since` is not
    /// given, or `full_state` is true).
    pub state: State,

    /// The private data that this user has attached to this room.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_data: Option<AccountData>,

    /// The ephemeral events in the room that aren't recorded in the timeline or state of the
    /// room. e.g. typing.
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
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Timeline {
    /// True if the number of events returned was limited by the `limit` on the filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limited: Option<bool>,

    /// A token that can be supplied to to the `from` parameter of the
    /// `/rooms/{roomId}/messages` endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_batch: Option<String>,

    /// A list of events.
    pub events: Vec<EventJson<RoomEvent>>,
}

/// State events in the room.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct State {
    /// A list of state events.
    pub events: Vec<EventJson<StateEvent>>,
}

/// The private data that this user has attached to this room.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AccountData {
    /// A list of events.
    pub events: Vec<EventJson<NonRoomEvent>>,
}

impl AccountData {
    fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Ephemeral events not recorded in the timeline or state of the room.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ephemeral {
    /// A list of events.
    pub events: Vec<EventJson<NonRoomEvent>>,
}

/// Information about room for rendering to clients.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoomSummary {
    /// Users which can be used to generate a room name if the room does not have
    /// one. Required if room name or canonical aliases are not set or empty.
    #[serde(rename = "m.heroes", default, skip_serializing_if = "Vec::is_empty")]
    pub heroes: Vec<String>,

    /// Number of users whose membership status is `join`.
    /// Required if field has changed since last sync; otherwise, it may be
    /// omitted.
    #[serde(
        rename = "m.joined_member_count",
        skip_serializing_if = "Option::is_none"
    )]
    pub joined_member_count: Option<UInt>,

    /// Number of users whose membership status is `invite`.
    /// Required if field has changed since last sync; otherwise, it may be
    /// omitted.
    #[serde(
        rename = "m.invited_member_count",
        skip_serializing_if = "Option::is_none"
    )]
    pub invited_member_count: Option<UInt>,
}

/// Updates to the rooms that the user has been invited to.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InvitedRoom {
    /// The state of a room that the user has been invited to.
    pub invite_state: InviteState,
}

/// The state of a room that the user has been invited to.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InviteState {
    /// A list of state events.
    pub events: Vec<EventJson<AnyStrippedStateEvent>>,
}

/// Updates to the presence status of other users.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Presence {
    /// A list of events.
    pub events: Vec<EventJson<PresenceEvent>>,
}

impl Presence {
    fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Messages sent dirrectly between devices.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ToDevice {
    /// A list of to-device events.
    pub events: Vec<EventJson<AnyToDeviceEvent>>,
}

impl ToDevice {
    fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Information on E2E device udpates.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeviceLists {
    /// List of users who have updated their device identity keys or who now
    /// share an encrypted room with the client since the previous sync
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed: Vec<String>,

    /// List of users who no longer share encrypted rooms since the previous sync
    /// response.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub left: Vec<String>,
}

#[cfg(test)]
mod tests {
    use std::{convert::TryInto, time::Duration};

    use super::{Filter, Request, SetPresence};

    #[test]
    fn serialize_sync_request() {
        let req: http::Request<Vec<u8>> = Request {
            filter: Some(Filter::FilterId("66696p746572".into())),
            since: Some("s72594_4483_1934".into()),
            full_state: true,
            set_presence: SetPresence::Offline,
            timeout: Some(Duration::from_millis(30000)),
        }
        .try_into()
        .unwrap();

        let uri = req.uri();
        let query = uri.query().unwrap();

        assert_eq!(uri.path(), "/_matrix/client/r0/sync");
        assert!(query.contains("filter=66696p746572"));
        assert!(query.contains("since=s72594_4483_1934"));
        assert!(query.contains("full_state=true"));
        assert!(query.contains("set_presence=offline"));
        assert!(query.contains("timeout=30000"))
    }

    #[test]
    fn deserialize_sync_request_with_query_params() {
        let uri = http::Uri::builder()
            .scheme("https")
            .authority("matrix.org")
            .path_and_query("/_matrix/client/r0/sync?filter=myfilter&since=myts&full_state=false&set_presence=offline&timeout=5000")
            .build()
            .unwrap();

        let req: Request = http::Request::builder()
            .uri(uri)
            .body(Vec::<u8>::new())
            .unwrap()
            .try_into()
            .unwrap();

        match req.filter {
            Some(Filter::FilterId(id)) if id == "myfilter" => {}
            _ => {
                panic!("Not the expected filter ID.");
            }
        }
        assert_eq!(req.since, Some("myts".into()));
        assert_eq!(req.full_state, false);
        assert_eq!(req.set_presence, SetPresence::Offline);
        assert_eq!(req.timeout, Some(Duration::from_millis(5000)));
    }

    #[test]
    fn deserialize_sync_request_without_query_params() {
        let uri = http::Uri::builder()
            .scheme("https")
            .authority("matrix.org")
            .path_and_query("/_matrix/client/r0/sync")
            .build()
            .unwrap();

        let req: Request = http::Request::builder()
            .uri(uri)
            .body(Vec::<u8>::new())
            .unwrap()
            .try_into()
            .unwrap();

        assert!(req.filter.is_none());
        assert!(req.since.is_none());
        assert_eq!(req.full_state, false);
        assert_eq!(req.set_presence, SetPresence::Online);
        assert!(req.timeout.is_none());
    }
}
