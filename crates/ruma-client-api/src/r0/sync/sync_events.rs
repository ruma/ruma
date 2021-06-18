//! [GET /_matrix/client/r0/sync](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-sync)

// FIXME: once https://github.com/rust-lang/rust/issues/84332 is resolved
// the structs can just be non_exhaustive (remove __test_exhaustive)
#![allow(clippy::exhaustive_structs)]

use std::{collections::BTreeMap, time::Duration};

use js_int::UInt;
use ruma_api::ruma_api;
use ruma_common::presence::PresenceState;
use ruma_events::{
    presence::PresenceEvent, AnyGlobalAccountDataEvent, AnyRoomAccountDataEvent,
    AnyStrippedStateEvent, AnySyncEphemeralRoomEvent, AnySyncRoomEvent, AnySyncStateEvent,
    AnyToDeviceEvent,
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
        #[serde(default, skip_serializing_if = "GlobalAccountData::is_empty")]
        pub account_data: GlobalAccountData,

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

        #[cfg(not(feature = "unstable-exhaustive-types"))]
        #[doc(hidden)]
        #[serde(skip, default = "crate::private")]
        pub __test_exhaustive: crate::Private,
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
    /// Creates a new `Response` with the given batch token.
    pub fn new(next_batch: String) -> Self {
        Self {
            next_batch,
            rooms: Default::default(),
            presence: Default::default(),
            account_data: Default::default(),
            to_device: Default::default(),
            device_lists: Default::default(),
            device_one_time_keys_count: BTreeMap::new(),
            #[cfg(not(feature = "unstable-exhaustive-types"))]
            __test_exhaustive: crate::private(),
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
#[derive(Clone, Debug, Deserialize, Serialize)]
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

    #[cfg(not(feature = "unstable-exhaustive-types"))]
    #[doc(hidden)]
    #[serde(skip, default = "crate::private")]
    pub __test_exhaustive: crate::Private,
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

impl Default for Rooms {
    fn default() -> Self {
        Self {
            leave: BTreeMap::new(),
            join: BTreeMap::new(),
            invite: BTreeMap::new(),
            #[cfg(not(feature = "unstable-exhaustive-types"))]
            __test_exhaustive: crate::private(),
        }
    }
}

/// Historical updates to left rooms.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeftRoom {
    /// The timeline of messages and state changes in the room up to the point when the user
    /// left.
    #[serde(default, skip_serializing_if = "Timeline::is_empty")]
    pub timeline: Timeline,

    /// The state updates for the room up to the start of the timeline.
    #[serde(default, skip_serializing_if = "State::is_empty")]
    pub state: State,

    /// The private data that this user has attached to this room.
    #[serde(default, skip_serializing_if = "RoomAccountData::is_empty")]
    pub account_data: RoomAccountData,

    #[cfg(not(feature = "unstable-exhaustive-types"))]
    #[doc(hidden)]
    #[serde(skip, default = "crate::private")]
    pub __test_exhaustive: crate::Private,
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

impl Default for LeftRoom {
    fn default() -> Self {
        Self {
            timeline: Default::default(),
            state: Default::default(),
            account_data: Default::default(),
            #[cfg(not(feature = "unstable-exhaustive-types"))]
            __test_exhaustive: crate::private(),
        }
    }
}

/// Updates to joined rooms.
#[derive(Clone, Debug, Deserialize, Serialize)]
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
    #[serde(default, skip_serializing_if = "RoomAccountData::is_empty")]
    pub account_data: RoomAccountData,

    /// The ephemeral events in the room that aren't recorded in the timeline or state of the
    /// room. e.g. typing.
    #[serde(default, skip_serializing_if = "Ephemeral::is_empty")]
    pub ephemeral: Ephemeral,

    #[cfg(not(feature = "unstable-exhaustive-types"))]
    #[doc(hidden)]
    #[serde(skip, default = "crate::private")]
    pub __test_exhaustive: crate::Private,
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

impl Default for JoinedRoom {
    fn default() -> Self {
        Self {
            summary: Default::default(),
            unread_notifications: Default::default(),
            timeline: Default::default(),
            state: Default::default(),
            account_data: Default::default(),
            ephemeral: Default::default(),
            #[cfg(not(feature = "unstable-exhaustive-types"))]
            __test_exhaustive: crate::private(),
        }
    }
}

/// Unread notifications count.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UnreadNotificationsCount {
    /// The number of unread notifications for this room with the highlight flag set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight_count: Option<UInt>,

    /// The total number of unread notifications for this room.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_count: Option<UInt>,

    #[cfg(not(feature = "unstable-exhaustive-types"))]
    #[doc(hidden)]
    #[serde(skip, default = "crate::private")]
    pub __test_exhaustive: crate::Private,
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

impl Default for UnreadNotificationsCount {
    fn default() -> Self {
        Self {
            highlight_count: None,
            notification_count: None,
            #[cfg(not(feature = "unstable-exhaustive-types"))]
            __test_exhaustive: crate::private(),
        }
    }
}

/// Events in the room.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Timeline {
    /// True if the number of events returned was limited by the `limit` on the filter.
    ///
    /// Default to `false`.
    #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
    pub limited: bool,

    /// A token that can be supplied to to the `from` parameter of the `/rooms/{roomId}/messages`
    /// endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_batch: Option<String>,

    /// A list of events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Raw<AnySyncRoomEvent>>,

    #[cfg(not(feature = "unstable-exhaustive-types"))]
    #[doc(hidden)]
    #[serde(skip, default = "crate::private")]
    pub __test_exhaustive: crate::Private,
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

impl Default for Timeline {
    fn default() -> Self {
        Self {
            limited: false,
            prev_batch: None,
            events: vec![],
            #[cfg(not(feature = "unstable-exhaustive-types"))]
            __test_exhaustive: crate::private(),
        }
    }
}

/// State events in the room.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct State {
    /// A list of state events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Raw<AnySyncStateEvent>>,

    #[cfg(not(feature = "unstable-exhaustive-types"))]
    #[doc(hidden)]
    #[serde(skip, default = "crate::private")]
    pub __test_exhaustive: crate::Private,
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

impl Default for State {
    fn default() -> Self {
        Self {
            events: vec![],
            #[cfg(not(feature = "unstable-exhaustive-types"))]
            __test_exhaustive: crate::private(),
        }
    }
}

/// The global private data created by this user.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GlobalAccountData {
    /// A list of events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Raw<AnyGlobalAccountDataEvent>>,

    #[cfg(not(feature = "unstable-exhaustive-types"))]
    #[doc(hidden)]
    #[serde(skip, default = "crate::private")]
    pub __test_exhaustive: crate::Private,
}

impl GlobalAccountData {
    /// Creates an empty `GlobalAccountData`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns true if there are no global account data updates.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl Default for GlobalAccountData {
    fn default() -> Self {
        Self {
            events: vec![],
            #[cfg(not(feature = "unstable-exhaustive-types"))]
            __test_exhaustive: crate::private(),
        }
    }
}

/// The private data that this user has attached to this room.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoomAccountData {
    /// A list of events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Raw<AnyRoomAccountDataEvent>>,

    #[cfg(not(feature = "unstable-exhaustive-types"))]
    #[doc(hidden)]
    #[serde(skip, default = "crate::private")]
    pub __test_exhaustive: crate::Private,
}

impl RoomAccountData {
    /// Creates an empty `RoomAccountData`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns true if there are no room account data updates.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl Default for RoomAccountData {
    fn default() -> Self {
        Self {
            events: vec![],
            #[cfg(not(feature = "unstable-exhaustive-types"))]
            __test_exhaustive: crate::private(),
        }
    }
}

/// Ephemeral events not recorded in the timeline or state of the room.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ephemeral {
    /// A list of events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Raw<AnySyncEphemeralRoomEvent>>,

    #[cfg(not(feature = "unstable-exhaustive-types"))]
    #[doc(hidden)]
    #[serde(skip, default = "crate::private")]
    pub __test_exhaustive: crate::Private,
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

impl Default for Ephemeral {
    fn default() -> Self {
        Self {
            events: vec![],
            #[cfg(not(feature = "unstable-exhaustive-types"))]
            __test_exhaustive: crate::private(),
        }
    }
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
    #[serde(rename = "m.joined_member_count", skip_serializing_if = "Option::is_none")]
    pub joined_member_count: Option<UInt>,

    /// Number of users whose membership status is `invite`.
    /// Required if field has changed since last sync; otherwise, it may be
    /// omitted.
    #[serde(rename = "m.invited_member_count", skip_serializing_if = "Option::is_none")]
    pub invited_member_count: Option<UInt>,

    #[cfg(not(feature = "unstable-exhaustive-types"))]
    #[doc(hidden)]
    #[serde(skip, default = "crate::private")]
    pub __test_exhaustive: crate::Private,
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

impl Default for RoomSummary {
    fn default() -> Self {
        Self {
            heroes: vec![],
            joined_member_count: None,
            invited_member_count: None,
            #[cfg(not(feature = "unstable-exhaustive-types"))]
            __test_exhaustive: crate::private(),
        }
    }
}

/// Updates to the rooms that the user has been invited to.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InvitedRoom {
    /// The state of a room that the user has been invited to.
    #[serde(default, skip_serializing_if = "InviteState::is_empty")]
    pub invite_state: InviteState,

    #[cfg(not(feature = "unstable-exhaustive-types"))]
    #[doc(hidden)]
    #[serde(skip, default = "crate::private")]
    pub __test_exhaustive: crate::Private,
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

impl Default for InvitedRoom {
    fn default() -> Self {
        Self {
            invite_state: Default::default(),
            #[cfg(not(feature = "unstable-exhaustive-types"))]
            __test_exhaustive: crate::private(),
        }
    }
}

/// The state of a room that the user has been invited to.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InviteState {
    /// A list of state events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Raw<AnyStrippedStateEvent>>,

    #[cfg(not(feature = "unstable-exhaustive-types"))]
    #[doc(hidden)]
    #[serde(skip, default = "crate::private")]
    pub __test_exhaustive: crate::Private,
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

impl Default for InviteState {
    fn default() -> Self {
        Self {
            events: vec![],
            #[cfg(not(feature = "unstable-exhaustive-types"))]
            __test_exhaustive: crate::private(),
        }
    }
}

/// Updates to the presence status of other users.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Presence {
    /// A list of events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Raw<PresenceEvent>>,

    #[cfg(not(feature = "unstable-exhaustive-types"))]
    #[doc(hidden)]
    #[serde(skip, default = "crate::private")]
    pub __test_exhaustive: crate::Private,
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

impl Default for Presence {
    fn default() -> Self {
        Self {
            events: vec![],
            #[cfg(not(feature = "unstable-exhaustive-types"))]
            __test_exhaustive: crate::private(),
        }
    }
}

/// Messages sent dirrectly between devices.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ToDevice {
    /// A list of to-device events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Raw<AnyToDeviceEvent>>,

    #[cfg(not(feature = "unstable-exhaustive-types"))]
    #[doc(hidden)]
    #[serde(skip, default = "crate::private")]
    pub __test_exhaustive: crate::Private,
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

impl Default for ToDevice {
    fn default() -> Self {
        Self {
            events: vec![],
            #[cfg(not(feature = "unstable-exhaustive-types"))]
            __test_exhaustive: crate::private(),
        }
    }
}

/// Information on E2E device udpates.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeviceLists {
    /// List of users who have updated their device identity keys or who now
    /// share an encrypted room with the client since the previous sync
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed: Vec<UserId>,

    /// List of users who no longer share encrypted rooms since the previous sync
    /// response.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub left: Vec<UserId>,

    #[cfg(not(feature = "unstable-exhaustive-types"))]
    #[doc(hidden)]
    #[serde(skip, default = "crate::private")]
    pub __test_exhaustive: crate::Private,
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

impl Default for DeviceLists {
    fn default() -> Self {
        Self {
            changed: vec![],
            left: vec![],
            #[cfg(not(feature = "unstable-exhaustive-types"))]
            __test_exhaustive: crate::private(),
        }
    }
}

#[cfg(test)]
mod tests {
    use assign::assign;
    use matches::assert_matches;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::Timeline;

    #[test]
    fn timeline_serde() {
        let timeline = assign!(Timeline::new(), { limited: true });
        let timeline_serialized = json!({ "limited": true });
        assert_eq!(to_json_value(timeline).unwrap(), timeline_serialized);

        let timeline_deserialized = from_json_value(timeline_serialized);
        assert_matches!(timeline_deserialized, Ok(Timeline { limited: true, .. }));

        let timeline_default = Timeline::default();
        assert_eq!(to_json_value(timeline_default).unwrap(), json!({}));

        let timeline_default_deserialized = from_json_value(json!({}));
        assert_matches!(timeline_default_deserialized, Ok(Timeline { limited: false, .. }));
    }
}

#[cfg(all(test, feature = "client"))]
mod client_tests {
    use std::time::Duration;

    use ruma_api::{OutgoingRequest as _, SendAccessToken};

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
        .try_into_http_request("https://homeserver.tld", SendAccessToken::IfRequired("auth_tok"))
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
        assert!(!req.full_state);
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
        assert!(!req.full_state);
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
        assert!(!req.full_state);
        assert_eq!(req.set_presence, PresenceState::Online);
        assert_eq!(req.timeout, Some(Duration::from_millis(0)));
    }
}
