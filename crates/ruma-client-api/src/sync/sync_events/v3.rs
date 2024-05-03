//! `/v3/` ([spec])
//!
//! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3sync

use std::{collections::BTreeMap, time::Duration};

use js_int::UInt;
use ruma_common::{
    api::{request, response, Metadata},
    metadata,
    presence::PresenceState,
    serde::Raw,
    DeviceKeyAlgorithm, OwnedEventId, OwnedRoomId,
};
use ruma_events::{
    presence::PresenceEvent, AnyGlobalAccountDataEvent, AnyRoomAccountDataEvent,
    AnyStrippedStateEvent, AnySyncEphemeralRoomEvent, AnySyncStateEvent, AnySyncTimelineEvent,
    AnyToDeviceEvent,
};
use serde::{Deserialize, Serialize};

use super::{DeviceLists, UnreadNotificationsCount};
use crate::filter::FilterDefinition;

const METADATA: Metadata = metadata! {
    method: GET,
    rate_limited: false,
    authentication: AccessToken,
    history: {
        1.0 => "/_matrix/client/r0/sync",
        1.1 => "/_matrix/client/v3/sync",
    }
};

/// Request type for the `sync` endpoint.
#[request(error = crate::Error)]
#[derive(Default)]
pub struct Request {
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
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    #[ruma_api(query)]
    pub full_state: bool,

    /// Controls whether the client is automatically marked as online by polling this API.
    ///
    /// Defaults to `PresenceState::Online`.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    #[ruma_api(query)]
    pub set_presence: PresenceState,

    /// The maximum time to poll in milliseconds before returning this request.
    #[serde(
        with = "ruma_common::serde::duration::opt_ms",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    #[ruma_api(query)]
    pub timeout: Option<Duration>,
}

/// Response type for the `sync` endpoint.
#[response(error = crate::Error)]
pub struct Response {
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

    /// Messages sent directly between devices.
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

    /// For each key algorithm, the number of unclaimed one-time keys
    /// currently held on the server for a device.
    ///
    /// The presence of this field indicates that the server supports
    /// fallback keys.
    pub device_unused_fallback_key_types: Option<Vec<DeviceKeyAlgorithm>>,
}

impl Request {
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
            device_unused_fallback_key_types: None,
        }
    }
}

/// A filter represented either as its full JSON definition or the ID of a saved filter.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::large_enum_variant)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(untagged)]
pub enum Filter {
    // The filter definition needs to be (de)serialized twice because it is a URL-encoded JSON
    // string. Since #[ruma_api(query)] only does the latter and this is a very uncommon
    // setup, we implement it through custom serde logic for this specific enum variant rather
    // than adding another ruma_api attribute.
    //
    // On the deserialization side, because this is an enum with #[serde(untagged)], serde
    // will try the variants in order (https://serde.rs/enum-representations.html). That means because
    // FilterDefinition is the first variant, JSON decoding is attempted first which is almost
    // functionally equivalent to looking at whether the first symbol is a '{' as the spec
    // says. (there are probably some corner cases like leading whitespace)
    /// A complete filter definition serialized to JSON.
    #[serde(with = "ruma_common::serde::json_string")]
    FilterDefinition(FilterDefinition),

    /// The ID of a filter saved on the server.
    FilterId(String),
}

impl From<FilterDefinition> for Filter {
    fn from(def: FilterDefinition) -> Self {
        Self::FilterDefinition(def)
    }
}

impl From<String> for Filter {
    fn from(id: String) -> Self {
        Self::FilterId(id)
    }
}

/// Updates to rooms.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Rooms {
    /// The rooms that the user has left or been banned from.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub leave: BTreeMap<OwnedRoomId, LeftRoom>,

    /// The rooms that the user has joined.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub join: BTreeMap<OwnedRoomId, JoinedRoom>,

    /// The rooms that the user has been invited to.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub invite: BTreeMap<OwnedRoomId, InvitedRoom>,

    /// The rooms that the user has knocked on.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub knock: BTreeMap<OwnedRoomId, KnockedRoom>,
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
    #[serde(default, skip_serializing_if = "RoomAccountData::is_empty")]
    pub account_data: RoomAccountData,
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

    /// Counts of [unread notifications] for this room.
    ///
    /// If `unread_thread_notifications` was set to `true` in the [`RoomEventFilter`], these
    /// include only the unread notifications for the main timeline.
    ///
    /// [unread notifications]: https://spec.matrix.org/latest/client-server-api/#receiving-notifications
    /// [`RoomEventFilter`]: crate::filter::RoomEventFilter
    #[serde(default, skip_serializing_if = "UnreadNotificationsCount::is_empty")]
    pub unread_notifications: UnreadNotificationsCount,

    /// Counts of [unread notifications] for threads in this room.
    ///
    /// This is a map from thread root ID to unread notifications in the thread.
    ///
    /// Only set if `unread_thread_notifications` was set to `true` in the [`RoomEventFilter`].
    ///
    /// [unread notifications]: https://spec.matrix.org/latest/client-server-api/#receiving-notifications
    /// [`RoomEventFilter`]: crate::filter::RoomEventFilter
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub unread_thread_notifications: BTreeMap<OwnedEventId, UnreadNotificationsCount>,

    /// The timeline of messages and state changes in the room.
    #[serde(default, skip_serializing_if = "Timeline::is_empty")]
    pub timeline: Timeline,

    /// Updates to the state, between the time indicated by the `since` parameter, and the
    /// start of the `timeline` (or all state up to the start of the `timeline`, if
    /// `since` is not given, or `full_state` is true).
    #[serde(default, skip_serializing_if = "State::is_empty")]
    pub state: State,

    /// The private data that this user has attached to this room.
    #[serde(default, skip_serializing_if = "RoomAccountData::is_empty")]
    pub account_data: RoomAccountData,

    /// The ephemeral events in the room that aren't recorded in the timeline or state of the
    /// room.
    #[serde(default, skip_serializing_if = "Ephemeral::is_empty")]
    pub ephemeral: Ephemeral,

    /// The number of unread events since the latest read receipt.
    ///
    /// This uses the unstable prefix in [MSC2654].
    ///
    /// [MSC2654]: https://github.com/matrix-org/matrix-spec-proposals/pull/2654
    #[cfg(feature = "unstable-msc2654")]
    #[serde(rename = "org.matrix.msc2654.unread_count", skip_serializing_if = "Option::is_none")]
    pub unread_count: Option<UInt>,
}

impl JoinedRoom {
    /// Creates an empty `JoinedRoom`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns true if there are no updates in the room.
    pub fn is_empty(&self) -> bool {
        let is_empty = self.summary.is_empty()
            && self.unread_notifications.is_empty()
            && self.unread_thread_notifications.is_empty()
            && self.timeline.is_empty()
            && self.state.is_empty()
            && self.account_data.is_empty()
            && self.ephemeral.is_empty();

        #[cfg(not(feature = "unstable-msc2654"))]
        return is_empty;

        #[cfg(feature = "unstable-msc2654")]
        return is_empty && self.unread_count.is_none();
    }
}

/// Updates to knocked rooms.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct KnockedRoom {
    /// The knock state.
    pub knock_state: KnockState,
}

/// A mapping from a key `events` to a list of `StrippedStateEvent`.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct KnockState {
    /// The list of events.
    pub events: Vec<Raw<AnyStrippedStateEvent>>,
}

/// Events in the room.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Timeline {
    /// True if the number of events returned was limited by the `limit` on the filter.
    ///
    /// Default to `false`.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    pub limited: bool,

    /// A token that can be supplied to to the `from` parameter of the
    /// `/rooms/{roomId}/messages` endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_batch: Option<String>,

    /// A list of events.
    pub events: Vec<Raw<AnySyncTimelineEvent>>,
}

impl Timeline {
    /// Creates an empty `Timeline`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns true if there are no timeline updates.
    ///
    /// A `Timeline` is considered non-empty if it has at least one event, a
    /// `prev_batch` value, or `limited` is `true`.
    pub fn is_empty(&self) -> bool {
        !self.limited && self.prev_batch.is_none() && self.events.is_empty()
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

    /// Creates a `State` with events
    pub fn with_events(events: Vec<Raw<AnySyncStateEvent>>) -> Self {
        State { events, ..Default::default() }
    }
}

impl From<Vec<Raw<AnySyncStateEvent>>> for State {
    fn from(events: Vec<Raw<AnySyncStateEvent>>) -> Self {
        State::with_events(events)
    }
}

/// The global private data created by this user.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct GlobalAccountData {
    /// A list of events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Raw<AnyGlobalAccountDataEvent>>,
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

/// The private data that this user has attached to this room.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RoomAccountData {
    /// A list of events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Raw<AnyRoomAccountDataEvent>>,
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
    /// Users which can be used to generate a room name if the room does not have one.
    ///
    /// Required if room name or canonical aliases are not set or empty.
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

impl From<InviteState> for InvitedRoom {
    fn from(invite_state: InviteState) -> Self {
        InvitedRoom { invite_state, ..Default::default() }
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

impl From<Vec<Raw<AnyStrippedStateEvent>>> for InviteState {
    fn from(events: Vec<Raw<AnyStrippedStateEvent>>) -> Self {
        InviteState { events, ..Default::default() }
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

/// Messages sent directly between devices.
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

#[cfg(test)]
mod tests {
    use assign::assign;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::Timeline;

    #[test]
    fn timeline_serde() {
        let timeline = assign!(Timeline::new(), { limited: true });
        let timeline_serialized = json!({ "events": [], "limited": true });
        assert_eq!(to_json_value(timeline).unwrap(), timeline_serialized);

        let timeline_deserialized = from_json_value::<Timeline>(timeline_serialized).unwrap();
        assert!(timeline_deserialized.limited);

        let timeline_default = Timeline::default();
        assert_eq!(to_json_value(timeline_default).unwrap(), json!({ "events": [] }));

        let timeline_default_deserialized =
            from_json_value::<Timeline>(json!({ "events": [] })).unwrap();
        assert!(!timeline_default_deserialized.limited);
    }
}

#[cfg(all(test, feature = "client"))]
mod client_tests {
    use std::time::Duration;

    use ruma_common::api::{MatrixVersion, OutgoingRequest as _, SendAccessToken};

    use super::{Filter, PresenceState, Request};

    #[test]
    fn serialize_all_params() {
        let req: http::Request<Vec<u8>> = Request {
            filter: Some(Filter::FilterId("66696p746572".to_owned())),
            since: Some("s72594_4483_1934".to_owned()),
            full_state: true,
            set_presence: PresenceState::Offline,
            timeout: Some(Duration::from_millis(30000)),
        }
        .try_into_http_request(
            "https://homeserver.tld",
            SendAccessToken::IfRequired("auth_tok"),
            &[MatrixVersion::V1_1],
        )
        .unwrap();

        let uri = req.uri();
        let query = uri.query().unwrap();

        assert_eq!(uri.path(), "/_matrix/client/v3/sync");
        assert!(query.contains("filter=66696p746572"));
        assert!(query.contains("since=s72594_4483_1934"));
        assert!(query.contains("full_state=true"));
        assert!(query.contains("set_presence=offline"));
        assert!(query.contains("timeout=30000"));
    }
}

#[cfg(all(test, feature = "server"))]
mod server_tests {
    use std::time::Duration;

    use assert_matches2::assert_matches;
    use ruma_common::{api::IncomingRequest as _, presence::PresenceState};

    use super::{Filter, Request};

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

        let req = Request::try_from_http_request(
            http::Request::builder().uri(uri).body(&[] as &[u8]).unwrap(),
            &[] as &[String],
        )
        .unwrap();

        assert_matches!(req.filter, Some(Filter::FilterId(id)));
        assert_eq!(id, "myfilter");
        assert_eq!(req.since.as_deref(), Some("myts"));
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

        let req = Request::try_from_http_request(
            http::Request::builder().uri(uri).body(&[] as &[u8]).unwrap(),
            &[] as &[String],
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

        let req = Request::try_from_http_request(
            http::Request::builder().uri(uri).body(&[] as &[u8]).unwrap(),
            &[] as &[String],
        )
        .unwrap();

        assert_matches!(req.filter, Some(Filter::FilterId(id)));
        assert_eq!(id, "EOKFFmdZYF");
        assert_eq!(req.since, None);
        assert!(!req.full_state);
        assert_eq!(req.set_presence, PresenceState::Online);
        assert_eq!(req.timeout, Some(Duration::from_millis(0)));
    }
}
