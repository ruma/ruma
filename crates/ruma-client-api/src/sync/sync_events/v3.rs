//! `/v3/` ([spec])
//!
//! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3sync

use std::{collections::BTreeMap, time::Duration};

use as_variant::as_variant;
use js_int::UInt;
use ruma_common::{
    EventId, OneTimeKeyAlgorithm, RoomId, UserId,
    api::{auth_scheme::AccessToken, request, response},
    metadata,
    presence::PresenceState,
    serde::Raw,
};
use ruma_events::{
    AnyGlobalAccountDataEvent, AnyRoomAccountDataEvent, AnyStrippedStateEvent,
    AnySyncEphemeralRoomEvent, AnySyncStateEvent, AnySyncTimelineEvent, AnyToDeviceEvent,
    presence::PresenceEvent,
};
use serde::{Deserialize, Serialize};

mod response_serde;

use super::{DeviceLists, UnreadNotificationsCount};
use crate::filter::FilterDefinition;

metadata! {
    method: GET,
    rate_limited: false,
    authentication: AccessToken,
    history: {
        1.0 => "/_matrix/client/r0/sync",
        1.1 => "/_matrix/client/v3/sync",
    }
}

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

    /// Controls whether to receive state changes between the previous sync and the **start** of
    /// the timeline, or between the previous sync and the **end** of the timeline.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    #[ruma_api(query)]
    pub use_state_after: bool,
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
    pub device_one_time_keys_count: BTreeMap<OneTimeKeyAlgorithm, UInt>,

    /// The unused fallback key algorithms.
    ///
    /// The presence of this field indicates that the server supports
    /// fallback keys.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_unused_fallback_key_types: Option<Vec<OneTimeKeyAlgorithm>>,
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
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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

    /// The rooms that the user has knocked on.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub knock: BTreeMap<RoomId, KnockedRoom>,
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
#[derive(Clone, Debug, Default, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct LeftRoom {
    /// The timeline of messages and state changes in the room up to the point when the user
    /// left.
    #[serde(skip_serializing_if = "Timeline::is_empty")]
    pub timeline: Timeline,

    /// The state updates for the room up to the start of the timeline.
    #[serde(flatten, skip_serializing_if = "State::is_before_and_empty")]
    pub state: State,

    /// The private data that this user has attached to this room.
    #[serde(skip_serializing_if = "RoomAccountData::is_empty")]
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
#[derive(Clone, Debug, Default, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct JoinedRoom {
    /// Information about the room which clients may need to correctly render it
    /// to users.
    #[serde(skip_serializing_if = "RoomSummary::is_empty")]
    pub summary: RoomSummary,

    /// Counts of [unread notifications] for this room.
    ///
    /// If `unread_thread_notifications` was set to `true` in the [`RoomEventFilter`], these
    /// include only the unread notifications for the main timeline.
    ///
    /// [unread notifications]: https://spec.matrix.org/latest/client-server-api/#receiving-notifications
    /// [`RoomEventFilter`]: crate::filter::RoomEventFilter
    #[serde(skip_serializing_if = "UnreadNotificationsCount::is_empty")]
    pub unread_notifications: UnreadNotificationsCount,

    /// Counts of [unread notifications] for threads in this room.
    ///
    /// This is a map from thread root ID to unread notifications in the thread.
    ///
    /// Only set if `unread_thread_notifications` was set to `true` in the [`RoomEventFilter`].
    ///
    /// [unread notifications]: https://spec.matrix.org/latest/client-server-api/#receiving-notifications
    /// [`RoomEventFilter`]: crate::filter::RoomEventFilter
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub unread_thread_notifications: BTreeMap<EventId, UnreadNotificationsCount>,

    /// The timeline of messages and state changes in the room.
    #[serde(skip_serializing_if = "Timeline::is_empty")]
    pub timeline: Timeline,

    /// Updates to the state, between the time indicated by the `since` parameter, and the
    /// start of the `timeline` (or all state up to the start of the `timeline`, if
    /// `since` is not given, or `full_state` is true).
    #[serde(flatten, skip_serializing_if = "State::is_before_and_empty")]
    pub state: State,

    /// The private data that this user has attached to this room.
    #[serde(skip_serializing_if = "RoomAccountData::is_empty")]
    pub account_data: RoomAccountData,

    /// The ephemeral events in the room that aren't recorded in the timeline or state of the
    /// room.
    #[serde(skip_serializing_if = "Ephemeral::is_empty")]
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

/// Updates to a room that the user has knocked upon.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct KnockedRoom {
    /// Updates to the stripped state of the room.
    #[serde(default, skip_serializing_if = "KnockState::is_empty")]
    pub knock_state: KnockState,
}

impl KnockedRoom {
    /// Creates an empty `KnockedRoom`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Whether there are updates for this room.
    pub fn is_empty(&self) -> bool {
        self.knock_state.is_empty()
    }
}

impl From<KnockState> for KnockedRoom {
    fn from(knock_state: KnockState) -> Self {
        KnockedRoom { knock_state, ..Default::default() }
    }
}

/// Stripped state updates of a room that the user has knocked upon.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct KnockState {
    /// The stripped state of a room that the user has knocked upon.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Raw<AnyStrippedStateEvent>>,
}

impl KnockState {
    /// Creates an empty `KnockState`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Whether there are stripped state updates in this room.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Events in the room.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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

/// State changes in a room.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum State {
    /// The state changes between the previous sync and the **start** of the timeline.
    ///
    /// To get the full list of state changes since the previous sync, the state events in
    /// [`Timeline`] must be added to these events to update the local state.
    ///
    /// To get this variant, `use_state_after` must be set to `false` in the [`Request`], which is
    /// the default.
    #[serde(rename = "state")]
    Before(StateEvents),

    /// The state changes between the previous sync and the **end** of the timeline.
    ///
    /// This contains the full list of state changes since the previous sync. State events in
    /// [`Timeline`] must be ignored to update the local state.
    ///
    /// To get this variant, `use_state_after` must be set to `true` in the [`Request`].
    #[serde(rename = "state_after")]
    After(StateEvents),
}

impl State {
    /// Returns true if this is the `Before` variant and there are no state updates.
    fn is_before_and_empty(&self) -> bool {
        as_variant!(self, Self::Before).is_some_and(|state| state.is_empty())
    }

    /// Returns true if there are no state updates.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Before(state) => state.is_empty(),
            Self::After(state) => state.is_empty(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::Before(Default::default())
    }
}

/// State events in the room.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct StateEvents {
    /// A list of state events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Raw<AnySyncStateEvent>>,
}

impl StateEvents {
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
        Self { events, ..Default::default() }
    }
}

impl From<Vec<Raw<AnySyncStateEvent>>> for StateEvents {
    fn from(events: Vec<Raw<AnySyncStateEvent>>) -> Self {
        Self::with_events(events)
    }
}

/// The global private data created by this user.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RoomSummary {
    /// Users which can be used to generate a room name if the room does not have one.
    ///
    /// Required if room name or canonical aliases are not set or empty.
    #[serde(rename = "m.heroes", default, skip_serializing_if = "Vec::is_empty")]
    pub heroes: Vec<UserId>,

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
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
    use ruma_common::canonical_json::assert_to_canonical_json_eq;
    use serde_json::{from_value as from_json_value, json};

    use super::Timeline;

    #[test]
    fn timeline_serde() {
        let timeline = assign!(Timeline::new(), { limited: true });
        let timeline_serialized = json!({ "events": [], "limited": true });
        assert_to_canonical_json_eq!(timeline, timeline_serialized.clone());

        let timeline_deserialized = from_json_value::<Timeline>(timeline_serialized).unwrap();
        assert!(timeline_deserialized.limited);

        let timeline_default = Timeline::default();
        assert_to_canonical_json_eq!(timeline_default, json!({ "events": [] }));

        let timeline_default_deserialized =
            from_json_value::<Timeline>(json!({ "events": [] })).unwrap();
        assert!(!timeline_default_deserialized.limited);
    }
}

#[cfg(all(test, feature = "client"))]
mod client_tests {
    use std::{borrow::Cow, time::Duration};

    use assert_matches2::assert_matches;
    use ruma_common::{
        RoomVersionId,
        api::{
            IncomingResponse as _, MatrixVersion, OutgoingRequest as _, SupportedVersions,
            auth_scheme::SendAccessToken,
        },
        event_id, room_id, user_id,
    };
    use ruma_events::AnyStrippedStateEvent;
    use serde_json::{Value as JsonValue, json, to_vec as to_json_vec};

    use super::{Filter, PresenceState, Request, Response, State};

    fn sync_state_event() -> JsonValue {
        json!({
            "content": {
              "avatar_url": "mxc://example.org/SEsfnsuifSDFSSEF",
              "displayname": "Alice Margatroid",
              "membership": "join",
            },
            "event_id": "$143273582443PhrSn",
            "origin_server_ts": 1_432_735_824,
            "sender": "@alice:example.org",
            "state_key": "@alice:example.org",
            "type": "m.room.member",
            "unsigned": {
              "age": 1234,
              "membership": "join",
            },
        })
    }

    #[test]
    fn serialize_request_all_params() {
        let supported = SupportedVersions {
            versions: [MatrixVersion::V1_1].into(),
            features: Default::default(),
        };
        let req: http::Request<Vec<u8>> = Request {
            filter: Some(Filter::FilterId("66696p746572".to_owned())),
            since: Some("s72594_4483_1934".to_owned()),
            full_state: true,
            set_presence: PresenceState::Offline,
            timeout: Some(Duration::from_millis(30000)),
            use_state_after: true,
        }
        .try_into_http_request(
            "https://homeserver.tld",
            SendAccessToken::IfRequired("auth_tok"),
            Cow::Owned(supported),
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
        assert!(query.contains("use_state_after=true"));
    }

    #[test]
    fn deserialize_response_invite() {
        let creator = user_id!("@creator:localhost");
        let invitee = user_id!("@invitee:localhost");
        let room_id = room_id!("!privateroom:localhost");
        let event_id = event_id!("$invite");

        let body = json!({
            "next_batch": "a00",
            "rooms": {
                "invite": {
                    room_id.clone(): {
                        "invite_state": {
                            "events": [
                                {
                                    "content": {
                                        "room_version": "11",
                                    },
                                    "type": "m.room.create",
                                    "state_key": "",
                                    "sender": creator,
                                },
                                {
                                    "content": {
                                        "membership": "invite",
                                    },
                                    "type": "m.room.member",
                                    "state_key": invitee,
                                    "sender": creator,
                                    "origin_server_ts": 4_345_456,
                                    "event_id": event_id,
                                },
                            ],
                        },
                    },
                },
            },
        });
        let http_response = http::Response::new(to_json_vec(&body).unwrap());

        let response = Response::try_from_http_response(http_response).unwrap();
        assert_eq!(response.next_batch, "a00");
        let private_room = response.rooms.invite.get(&room_id).unwrap();

        let first_event = private_room.invite_state.events[0].deserialize().unwrap();
        assert_matches!(first_event, AnyStrippedStateEvent::RoomCreate(create_event));
        assert_eq!(create_event.sender, creator);
        assert_eq!(create_event.content.room_version, RoomVersionId::V11);
    }

    #[test]
    fn deserialize_response_no_state() {
        let joined_room_id = room_id!("!joined:localhost");
        let left_room_id = room_id!("!left:localhost");
        let event = sync_state_event();

        let body = json!({
            "next_batch": "aaa",
            "rooms": {
                "join": {
                    joined_room_id.clone(): {
                        "timeline": {
                            "events": [
                                event,
                            ],
                        },
                    },
                },
                "leave": {
                    left_room_id.clone(): {
                        "timeline": {
                            "events": [
                                event,
                            ],
                        },
                    },
                },
            },
        });

        let http_response = http::Response::new(to_json_vec(&body).unwrap());

        let response = Response::try_from_http_response(http_response).unwrap();
        assert_eq!(response.next_batch, "aaa");

        let joined_room = response.rooms.join.get(&joined_room_id).unwrap();
        assert_eq!(joined_room.timeline.events.len(), 1);
        assert!(joined_room.state.is_before_and_empty());

        let left_room = response.rooms.leave.get(&left_room_id).unwrap();
        assert_eq!(left_room.timeline.events.len(), 1);
        assert!(left_room.state.is_before_and_empty());
    }

    #[test]
    fn deserialize_response_state_before() {
        let joined_room_id = room_id!("!joined:localhost");
        let left_room_id = room_id!("!left:localhost");
        let event = sync_state_event();

        let body = json!({
            "next_batch": "aaa",
            "rooms": {
                "join": {
                    joined_room_id.clone(): {
                        "state": {
                            "events": [
                                event,
                            ],
                        },
                    },
                },
                "leave": {
                    left_room_id.clone(): {
                        "state": {
                            "events": [
                                event,
                            ],
                        },
                    },
                },
            },
        });

        let http_response = http::Response::new(to_json_vec(&body).unwrap());

        let response = Response::try_from_http_response(http_response).unwrap();
        assert_eq!(response.next_batch, "aaa");

        let joined_room = response.rooms.join.get(&joined_room_id).unwrap();
        assert!(joined_room.timeline.is_empty());
        assert_matches!(&joined_room.state, State::Before(state));
        assert_eq!(state.events.len(), 1);

        let left_room = response.rooms.leave.get(&left_room_id).unwrap();
        assert!(left_room.timeline.is_empty());
        assert_matches!(&left_room.state, State::Before(state));
        assert_eq!(state.events.len(), 1);
    }

    #[test]
    fn deserialize_response_empty_state_after() {
        let joined_room_id = room_id!("!joined:localhost");
        let left_room_id = room_id!("!left:localhost");

        let body = json!({
            "next_batch": "aaa",
            "rooms": {
                "join": {
                    joined_room_id.clone(): {
                        "state_after": {},
                    },
                },
                "leave": {
                    left_room_id.clone(): {
                        "state_after": {},
                    },
                },
            },
        });

        let http_response = http::Response::new(to_json_vec(&body).unwrap());

        let response = Response::try_from_http_response(http_response).unwrap();
        assert_eq!(response.next_batch, "aaa");

        let joined_room = response.rooms.join.get(&joined_room_id).unwrap();
        assert!(joined_room.timeline.is_empty());
        assert_matches!(&joined_room.state, State::After(state));
        assert_eq!(state.events.len(), 0);

        let left_room = response.rooms.leave.get(&left_room_id).unwrap();
        assert!(left_room.timeline.is_empty());
        assert_matches!(&left_room.state, State::After(state));
        assert_eq!(state.events.len(), 0);
    }

    #[test]
    fn deserialize_response_non_empty_state_after() {
        let joined_room_id = room_id!("!joined:localhost");
        let left_room_id = room_id!("!left:localhost");
        let event = sync_state_event();

        let body = json!({
            "next_batch": "aaa",
            "rooms": {
                "join": {
                    joined_room_id.clone(): {
                        "state_after": {
                            "events": [
                                event,
                            ],
                        },
                    },
                },
                "leave": {
                    left_room_id.clone(): {
                        "state_after": {
                            "events": [
                                event,
                            ],
                        },
                    },
                },
            },
        });

        let http_response = http::Response::new(to_json_vec(&body).unwrap());

        let response = Response::try_from_http_response(http_response).unwrap();
        assert_eq!(response.next_batch, "aaa");

        let joined_room = response.rooms.join.get(&joined_room_id).unwrap();
        assert!(joined_room.timeline.is_empty());
        assert_matches!(&joined_room.state, State::After(state));
        assert_eq!(state.events.len(), 1);

        let left_room = response.rooms.leave.get(&left_room_id).unwrap();
        assert!(left_room.timeline.is_empty());
        assert_matches!(&left_room.state, State::After(state));
        assert_eq!(state.events.len(), 1);
    }
}

#[cfg(all(test, feature = "server"))]
mod server_tests {
    use std::time::Duration;

    use assert_matches2::assert_matches;
    use ruma_common::{
        api::{IncomingRequest as _, OutgoingResponse as _},
        presence::PresenceState,
        room_id,
        serde::Raw,
    };
    use ruma_events::AnySyncStateEvent;
    use serde_json::{Value as JsonValue, from_slice as from_json_slice, json};

    use super::{Filter, JoinedRoom, LeftRoom, Request, Response, State};

    fn sync_state_event() -> Raw<AnySyncStateEvent> {
        Raw::new(&json!({
            "content": {
              "avatar_url": "mxc://example.org/SEsfnsuifSDFSSEF",
              "displayname": "Alice Margatroid",
              "membership": "join",
            },
            "event_id": "$143273582443PhrSn",
            "origin_server_ts": 1_432_735_824,
            "sender": "@alice:example.org",
            "state_key": "@alice:example.org",
            "type": "m.room.member",
            "unsigned": {
              "age": 1234,
              "membership": "join",
            },
        }))
        .unwrap()
        .cast_unchecked()
    }

    #[test]
    fn deserialize_request_all_query_params() {
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
    fn deserialize_request_no_query_params() {
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
    fn deserialize_request_some_query_params() {
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

    #[test]
    fn serialize_response_no_state() {
        let joined_room_id = room_id!("!joined:localhost");
        let left_room_id = room_id!("!left:localhost");
        let event = sync_state_event();

        let mut response = Response::new("aaa".to_owned());

        let mut joined_room = JoinedRoom::new();
        joined_room.timeline.events.push(event.clone().cast());
        response.rooms.join.insert(joined_room_id.clone(), joined_room);

        let mut left_room = LeftRoom::new();
        left_room.timeline.events.push(event.clone().cast());
        response.rooms.leave.insert(left_room_id.clone(), left_room);

        let http_response = response.try_into_http_response::<Vec<u8>>().unwrap();

        assert_eq!(
            from_json_slice::<JsonValue>(http_response.body()).unwrap(),
            json!({
                "next_batch": "aaa",
                "rooms": {
                    "join": {
                        joined_room_id: {
                            "timeline": {
                                "events": [
                                    event,
                                ],
                            },
                        },
                    },
                    "leave": {
                        left_room_id: {
                            "timeline": {
                                "events": [
                                    event,
                                ],
                            },
                        },
                    },
                },
            })
        );
    }

    #[test]
    fn serialize_response_state_before() {
        let joined_room_id = room_id!("!joined:localhost");
        let left_room_id = room_id!("!left:localhost");
        let event = sync_state_event();

        let mut response = Response::new("aaa".to_owned());

        let mut joined_room = JoinedRoom::new();
        joined_room.state = State::Before(vec![event.clone()].into());
        response.rooms.join.insert(joined_room_id.clone(), joined_room);

        let mut left_room = LeftRoom::new();
        left_room.state = State::Before(vec![event.clone()].into());
        response.rooms.leave.insert(left_room_id.clone(), left_room);

        let http_response = response.try_into_http_response::<Vec<u8>>().unwrap();

        assert_eq!(
            from_json_slice::<JsonValue>(http_response.body()).unwrap(),
            json!({
                "next_batch": "aaa",
                "rooms": {
                    "join": {
                        joined_room_id: {
                            "state": {
                                "events": [
                                    event,
                                ],
                            },
                        },
                    },
                    "leave": {
                        left_room_id: {
                            "state": {
                                "events": [
                                    event,
                                ],
                            },
                        },
                    },
                },
            })
        );
    }

    #[test]
    fn serialize_response_empty_state_after() {
        let joined_room_id = room_id!("!joined:localhost");
        let left_room_id = room_id!("!left:localhost");

        let mut response = Response::new("aaa".to_owned());

        let mut joined_room = JoinedRoom::new();
        joined_room.state = State::After(Default::default());
        response.rooms.join.insert(joined_room_id.clone(), joined_room);

        let mut left_room = LeftRoom::new();
        left_room.state = State::After(Default::default());
        response.rooms.leave.insert(left_room_id.clone(), left_room);

        let http_response = response.try_into_http_response::<Vec<u8>>().unwrap();

        assert_eq!(
            from_json_slice::<JsonValue>(http_response.body()).unwrap(),
            json!({
                "next_batch": "aaa",
                "rooms": {
                    "join": {
                        joined_room_id: {
                            "state_after": {},
                        },
                    },
                    "leave": {
                        left_room_id: {
                            "state_after": {},
                        },
                    },
                },
            })
        );
    }

    #[test]
    fn serialize_response_non_empty_state_after() {
        let joined_room_id = room_id!("!joined:localhost");
        let left_room_id = room_id!("!left:localhost");
        let event = sync_state_event();

        let mut response = Response::new("aaa".to_owned());

        let mut joined_room = JoinedRoom::new();
        joined_room.state = State::After(vec![event.clone()].into());
        response.rooms.join.insert(joined_room_id.clone(), joined_room);

        let mut left_room = LeftRoom::new();
        left_room.state = State::After(vec![event.clone()].into());
        response.rooms.leave.insert(left_room_id.clone(), left_room);

        let http_response = response.try_into_http_response::<Vec<u8>>().unwrap();

        assert_eq!(
            from_json_slice::<JsonValue>(http_response.body()).unwrap(),
            json!({
                "next_batch": "aaa",
                "rooms": {
                    "join": {
                        joined_room_id: {
                            "state_after": {
                                "events": [
                                    event,
                                ],
                            },
                        },
                    },
                    "leave": {
                        left_room_id: {
                            "state_after": {
                                "events": [
                                    event,
                                ],
                            },
                        },
                    },
                },
            })
        );
    }
}
