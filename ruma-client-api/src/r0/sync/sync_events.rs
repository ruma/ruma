//! [GET /_matrix/client/r0/sync](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-sync)

use std::{collections::BTreeMap, time::Duration};

use js_int::UInt;
use ruma_api::ruma_api;
use ruma_common::presence::PresenceState;
use ruma_events::{
    presence::PresenceEvent, AnyBasicEvent, AnyStrippedStateEvent, AnySyncEphemeralRoomEvent,
    AnySyncRoomEvent, AnySyncStateEvent, AnyToDeviceEvent,
};
use ruma_identifiers::{DeviceKeyAlgorithm, RoomId, UserId};
use ruma_serde::{Outgoing, Raw};
use serde::{Deserialize, Serialize};

use crate::r0::filter::{FilterDefinition, IncomingFilterDefinition};

ruma_api! {
    metadata: {
        description: "Get all new events from all rooms since the last sync or a given point of time.",
        method: GET,
        name: "sync",
        path: "/_matrix/client/r0/sync",
        rate_limited: false,
        authentication: AccessToken,
    }

    #[derive(Default)]
    request: {
        /// A filter represented either as its full JSON definition or the ID of a saved filter.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub filter: Option<&'a Filter<'a>>,

        /// A point in time to continue a sync from.
        ///
        /// Should be a token from the `next_batch` field of a previous `/sync`
        /// request.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub since: Option<&'a str>,

        /// Controls whether to include the full state for all rooms the user is a member of.
        #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
        #[ruma_api(query)]
        pub full_state: bool,

        /// Controls whether the client is automatically marked as online by polling this API.
        ///
        /// Defaults to `PresenceState::Online`.
        #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
        #[ruma_api(query)]
        pub set_presence: &'a PresenceState,

        /// The maximum time to poll in milliseconds before returning this request.
        #[serde(
            with = "ruma_serde::duration::opt_ms",
            default,
            skip_serializing_if = "Option::is_none",
        )]
        #[ruma_api(query)]
        pub timeout: Option<Duration>,
    }

    response: {
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
        #[serde(default, skip_serializing_if = "DeviceLists::is_empty")]
        pub device_lists: DeviceLists,

        /// For each key algorithm, the number of unclaimed one-time keys
        /// currently held on the server for a device.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub device_one_time_keys_count: BTreeMap<DeviceKeyAlgorithm, UInt>,
    }

    error: crate::Error
}

impl Request<'_> {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Default::default()
    }
}

impl Response {
    /// Creates a `Response` with the given batch token.
    pub fn new(next_batch: String) -> Self {
        Self {
            next_batch,
            rooms: Default::default(),
            presence: Default::default(),
            account_data: Default::default(),
            to_device: Default::default(),
            device_lists: Default::default(),
            device_one_time_keys_count: BTreeMap::new(),
        }
    }
}

/// A filter represented either as its full JSON definition or the ID of a saved filter.
#[derive(Clone, Debug, Outgoing, Serialize)]
#[allow(clippy::large_enum_variant)]
#[serde(untagged)]
pub enum Filter<'a> {
    // The filter definition needs to be (de)serialized twice because it is a URL-encoded JSON
    // string. Since #[ruma_api(query)] only does the latter and this is a very uncommon
    // setup, we implement it through custom serde logic for this specific enum variant rather
    // than adding another ruma_api attribute.
    //
    // On the deserialization side, because this is an enum with #[serde(untagged)], serde will
    // try the variants in order (https://serde.rs/enum-representations.html). That means because
    // FilterDefinition is the first variant, JSON decoding is attempted first which is almost
    // functionally equivalent to looking at whether the first symbol is a '{' as the spec says.
    // (there are probably some corner cases like leading whitespace)
    #[serde(with = "ruma_serde::json_string")]
    /// A complete filter definition serialized to JSON.
    FilterDefinition(FilterDefinition<'a>),

    /// The ID of a filter saved on the server.
    FilterId(&'a str),
}

impl<'a> From<FilterDefinition<'a>> for Filter<'a> {
    fn from(def: FilterDefinition<'a>) -> Self {
        Self::FilterDefinition(def)
    }
}

impl<'a> From<&'a str> for Filter<'a> {
    fn from(id: &'a str) -> Self {
        Self::FilterId(id)
    }
}

/// Updates to rooms.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Rooms {
    /// The rooms that the user has left or been banned from.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub leave: BTreeMap<RoomId, LeftRoom>,

    /// The rooms that the user has joined.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub join: BTreeMap<RoomId, JoinedRoom>,

    /// The rooms that the user has been invited to.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub invite: BTreeMap<RoomId, InvitedRoom>,
}

impl Rooms {
    /// Creates an empty `Rooms`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns true if there is no update in any room.
    pub fn is_empty(&self) -> bool {
        self.leave.is_empty() && self.join.is_empty() && self.invite.is_empty()
    }
}

/// Historical updates to left rooms.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct LeftRoom {
    /// The timeline of messages and state changes in the room up to the point when the user
    /// left.
    #[serde(default, skip_serializing_if = "Timeline::is_empty")]
    pub timeline: Timeline,

    /// The state updates for the room up to the start of the timeline.
    #[serde(default, skip_serializing_if = "State::is_empty")]
    pub state: State,

    /// The private data that this user has attached to this room.
    #[serde(default, skip_serializing_if = "AccountData::is_empty")]
    pub account_data: AccountData,
}

impl LeftRoom {
    /// Creates an empty `LeftRoom`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns true if there are updates in the room.
    pub fn is_empty(&self) -> bool {
        self.timeline.is_empty() && self.state.is_empty() && self.account_data.is_empty()
    }
}

/// Updates to joined rooms.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct JoinedRoom {
    /// Information about the room which clients may need to correctly render it
    /// to users.
    #[serde(default, skip_serializing_if = "RoomSummary::is_empty")]
    pub summary: RoomSummary,

    /// Counts of unread notifications for this room.
    #[serde(default, skip_serializing_if = "UnreadNotificationsCount::is_empty")]
    pub unread_notifications: UnreadNotificationsCount,

    /// The timeline of messages and state changes in the room.
    #[serde(default, skip_serializing_if = "Timeline::is_empty")]
    pub timeline: Timeline,

    /// Updates to the state, between the time indicated by the `since` parameter, and the start
    /// of the `timeline` (or all state up to the start of the `timeline`, if `since` is not
    /// given, or `full_state` is true).
    #[serde(default, skip_serializing_if = "State::is_empty")]
    pub state: State,

    /// The private data that this user has attached to this room.
    #[serde(default, skip_serializing_if = "AccountData::is_empty")]
    pub account_data: AccountData,

    /// The ephemeral events in the room that aren't recorded in the timeline or state of the
    /// room. e.g. typing.
    #[serde(default, skip_serializing_if = "Ephemeral::is_empty")]
    pub ephemeral: Ephemeral,
}

impl JoinedRoom {
    /// Creates an empty `JoinedRoom`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns true if there are no updates in the room.
    pub fn is_empty(&self) -> bool {
        self.summary.is_empty()
            && self.unread_notifications.is_empty()
            && self.timeline.is_empty()
            && self.state.is_empty()
            && self.account_data.is_empty()
            && self.ephemeral.is_empty()
    }
}

/// unread notifications count
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct UnreadNotificationsCount {
    /// The number of unread notifications for this room with the highlight flag set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight_count: Option<UInt>,

    /// The total number of unread notifications for this room.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_count: Option<UInt>,
}

impl UnreadNotificationsCount {
    /// Creates an empty `UnreadNotificationsCount`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns true if there are no notification count updates.
    pub fn is_empty(&self) -> bool {
        self.highlight_count.is_none() && self.notification_count.is_none()
    }
}

/// Events in the room.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Timeline {
    /// True if the number of events returned was limited by the `limit` on the filter.
    #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
    pub limited: bool,

    /// A token that can be supplied to to the `from` parameter of the
    /// `/rooms/{roomId}/messages` endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_batch: Option<String>,

    /// A list of events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Raw<AnySyncRoomEvent>>,
}

impl Timeline {
    /// Creates an empty `Timeline`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns true if there are no timeline updates.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// State events in the room.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct State {
    /// A list of state events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Raw<AnySyncStateEvent>>,
}

impl State {
    /// Creates an empty `State`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns true if there are no state updates.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// The private data that this user has attached to this room.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct AccountData {
    /// A list of events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Raw<AnyBasicEvent>>,
}

impl AccountData {
    /// Creates an empty `AccountData`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns true if there are no account data updates.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Ephemeral events not recorded in the timeline or state of the room.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Ephemeral {
    /// A list of events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Raw<AnySyncEphemeralRoomEvent>>,
}

impl Ephemeral {
    /// Creates an empty `Ephemeral`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns true if there are no ephemeral event updates.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Information about room for rendering to clients.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RoomSummary {
    /// Users which can be used to generate a room name if the room does not have
    /// one. Required if room name or canonical aliases are not set or empty.
    #[serde(rename = "m.heroes", default, skip_serializing_if = "Vec::is_empty")]
    pub heroes: Vec<String>,

    /// Number of users whose membership status is `join`.
    /// Required if field has changed since last sync; otherwise, it may be
    /// omitted.
    #[serde(rename = "m.joined_member_count", skip_serializing_if = "Option::is_none")]
    pub joined_member_count: Option<UInt>,

    /// Number of users whose membership status is `invite`.
    /// Required if field has changed since last sync; otherwise, it may be
    /// omitted.
    #[serde(rename = "m.invited_member_count", skip_serializing_if = "Option::is_none")]
    pub invited_member_count: Option<UInt>,
}

impl RoomSummary {
    /// Creates an empty `RoomSummary`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns true if there are no room summary updates.
    pub fn is_empty(&self) -> bool {
        self.heroes.is_empty()
            && self.joined_member_count.is_none()
            && self.invited_member_count.is_none()
    }
}

/// Updates to the rooms that the user has been invited to.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct InvitedRoom {
    /// The state of a room that the user has been invited to.
    #[serde(default, skip_serializing_if = "InviteState::is_empty")]
    pub invite_state: InviteState,
}

impl InvitedRoom {
    /// Creates an empty `InvitedRoom`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns true if there are no updates to this room.
    pub fn is_empty(&self) -> bool {
        self.invite_state.is_empty()
    }
}

/// The state of a room that the user has been invited to.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct InviteState {
    /// A list of state events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Raw<AnyStrippedStateEvent>>,
}

impl InviteState {
    /// Creates an empty `InviteState`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns true if there are no state updates.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Updates to the presence status of other users.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Presence {
    /// A list of events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Raw<PresenceEvent>>,
}

impl Presence {
    /// Creates an empty `Presence`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns true if there are no presence updates.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Messages sent dirrectly between devices.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ToDevice {
    /// A list of to-device events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Raw<AnyToDeviceEvent>>,
}

impl ToDevice {
    /// Creates an empty `ToDevice`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns true if there are no to-device events.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Information on E2E device udpates.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct DeviceLists {
    /// List of users who have updated their device identity keys or who now
    /// share an encrypted room with the client since the previous sync
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed: Vec<UserId>,

    /// List of users who no longer share encrypted rooms since the previous sync
    /// response.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub left: Vec<UserId>,
}

impl DeviceLists {
    /// Creates an empty `DeviceLists`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns true if there are no device list updates.
    pub fn is_empty(&self) -> bool {
        self.changed.is_empty() && self.left.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::Timeline;

    #[test]
    fn timeline_serde() {
        let timeline = Timeline { limited: true, prev_batch: None, events: vec![] };

        let timeline_serialized = json!({
            "limited": true,
        });

        assert_eq!(to_json_value(timeline).unwrap(), timeline_serialized);

        let timeline_deserialized: Timeline = from_json_value(timeline_serialized).unwrap();
        assert_eq!(timeline_deserialized.limited, true);

        let timeline_default = Timeline::default();

        let timeline_default_serialized = json!({});

        assert_eq!(to_json_value(timeline_default).unwrap(), timeline_default_serialized);

        let timeline_default_deserialized: Timeline =
            from_json_value(timeline_default_serialized).unwrap();
        assert_eq!(timeline_default_deserialized.limited, false);
    }
}

#[cfg(all(test, feature = "client"))]
mod client_tests {
    use std::time::Duration;

    use ruma_api::OutgoingRequest as _;

    use super::{Filter, PresenceState, Request};

    #[test]
    fn serialize_all_params() {
        let req: http::Request<Vec<u8>> = Request {
            filter: Some(&Filter::FilterId("66696p746572")),
            since: Some("s72594_4483_1934"),
            full_state: true,
            set_presence: &PresenceState::Offline,
            timeout: Some(Duration::from_millis(30000)),
        }
        .try_into_http_request("https://homeserver.tld", Some("auth_tok"))
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
}

#[cfg(all(test, feature = "server"))]
mod server_tests {
    use std::time::Duration;

    use matches::assert_matches;
    use ruma_api::IncomingRequest as _;
    use ruma_common::presence::PresenceState;

    use super::{IncomingFilter, IncomingRequest};

    #[test]
    fn deserialize_all_query_params() {
        let uri = http::Uri::builder()
            .scheme("https")
            .authority("matrix.org")
            .path_and_query(
                "/_matrix/client/r0/sync\
                 ?filter=myfilter\
                 &since=myts\
                 &full_state=false\
                 &set_presence=offline\
                 &timeout=5000",
            )
            .build()
            .unwrap();

        let req = IncomingRequest::try_from_http_request(
            http::Request::builder().uri(uri).body(&[] as &[u8]).unwrap(),
        )
        .unwrap();

        assert_matches!(req.filter, Some(IncomingFilter::FilterId(id)) if id == "myfilter");
        assert_eq!(req.since, Some("myts".into()));
        assert_eq!(req.full_state, false);
        assert_eq!(req.set_presence, PresenceState::Offline);
        assert_eq!(req.timeout, Some(Duration::from_millis(5000)));
    }

    #[test]
    fn deserialize_no_query_params() {
        let uri = http::Uri::builder()
            .scheme("https")
            .authority("matrix.org")
            .path_and_query("/_matrix/client/r0/sync")
            .build()
            .unwrap();

        let req = IncomingRequest::try_from_http_request(
            http::Request::builder().uri(uri).body(&[] as &[u8]).unwrap(),
        )
        .unwrap();

        assert_matches!(req.filter, None);
        assert_eq!(req.since, None);
        assert_eq!(req.full_state, false);
        assert_eq!(req.set_presence, PresenceState::Online);
        assert_eq!(req.timeout, None);
    }

    #[test]
    fn deserialize_some_query_params() {
        let uri = http::Uri::builder()
            .scheme("https")
            .authority("matrix.org")
            .path_and_query(
                "/_matrix/client/r0/sync\
                 ?filter=EOKFFmdZYF\
                 &timeout=0",
            )
            .build()
            .unwrap();

        let req = IncomingRequest::try_from_http_request(
            http::Request::builder().uri(uri).body(&[] as &[u8]).unwrap(),
        )
        .unwrap();

        assert_matches!(req.filter, Some(IncomingFilter::FilterId(id)) if id == "EOKFFmdZYF");
        assert_eq!(req.since, None);
        assert_eq!(req.full_state, false);
        assert_eq!(req.set_presence, PresenceState::Online);
        assert_eq!(req.timeout, Some(Duration::from_millis(0)));
    }
}
