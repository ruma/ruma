//! `POST /_matrix/client/unstable/org.matrix.msc3575/sync` ([MSC])
//!
//! Get all new events in a sliding window of rooms since the last sync or a given point in time.
//!
//! [MSC]: https://github.com/matrix-org/matrix-doc/blob/kegan/sync-v3/proposals/3575-sync.md

use std::{collections::BTreeMap, time::Duration};

use js_int::UInt;
use js_option::JsOption;
use ruma_common::{
    api::{request, response, Metadata},
    metadata,
    serde::{deserialize_cow_str, duration::opt_ms, Raw},
    DeviceKeyAlgorithm, MilliSecondsSinceUnixEpoch, OwnedMxcUri, OwnedRoomId, OwnedUserId, RoomId,
};
use ruma_events::{
    receipt::SyncReceiptEvent, typing::SyncTypingEvent, AnyGlobalAccountDataEvent,
    AnyRoomAccountDataEvent, AnyStrippedStateEvent, AnySyncStateEvent, AnySyncTimelineEvent,
    AnyToDeviceEvent, StateEventType, TimelineEventType,
};
use serde::{de::Error as _, Deserialize, Serialize};

use super::{DeviceLists, UnreadNotificationsCount};

const METADATA: Metadata = metadata! {
    method: POST,
    rate_limited: false,
    authentication: AccessToken,
    history: {
        unstable => "/_matrix/client/unstable/org.matrix.msc3575/sync",
        // 1.4 => "/_matrix/client/v4/sync",
    }
};

/// Request type for the `sync` endpoint.
#[request(error = crate::Error)]
#[derive(Default)]
pub struct Request {
    /// A point in time to continue a sync from.
    ///
    /// Should be a token from the `pos` field of a previous `/sync`
    /// response.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ruma_api(query)]
    pub pos: Option<String>,

    /// The delta token to store for session recovery.
    ///
    /// The delta token is a future bandwidth optimisation to resume from an
    /// earlier session. If you received a delta token in your last response
    /// you can persist and it when establishing a new sessions to "resume"
    /// from the last state and not resend information you had stored. If you
    /// send a delta token, the server expects you to have stored the last
    /// state, if there is no delta token present the server will resend all
    /// information necessary to calculate the state.
    ///
    /// Please consult ["Bandwidth optimisations for persistent clients" of the MSC][MSC]
    /// for further details, expectations of the implementation and limitations
    /// to consider before implementing this.
    ///
    /// [MSC]: https://github.com/matrix-org/matrix-spec-proposals/blob/kegan/sync-v3/proposals/3575-sync.md#bandwidth-optimisations-for-persistent-clients
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta_token: Option<String>,

    /// A unique string identifier for this connection to the server.
    ///
    /// Optional. If this is missing, only one sliding sync connection can be made to the server at
    /// any one time. Clients need to set this to allow more than one connection concurrently,
    /// so the server can distinguish between connections. This is NOT STICKY and must be
    /// provided with every request, if your client needs more than one concurrent connection.
    ///
    /// Limitation: it must not contain more than 16 chars, due to it being required with every
    /// request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conn_id: Option<String>,

    /// Allows clients to know what request params reached the server,
    /// functionally similar to txn IDs on /send for events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub txn_id: Option<String>,

    /// The maximum time to poll before responding to this request.
    #[serde(with = "opt_ms", default, skip_serializing_if = "Option::is_none")]
    #[ruma_api(query)]
    pub timeout: Option<Duration>,

    /// The list configurations of rooms we are interested in mapped by
    /// name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub lists: BTreeMap<String, SyncRequestList>,

    /// Specific rooms and event types that we want to receive events from.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub room_subscriptions: BTreeMap<OwnedRoomId, RoomSubscription>,

    /// Specific rooms we no longer want to receive events from.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub unsubscribe_rooms: Vec<OwnedRoomId>,

    /// Extensions API.
    #[serde(default, skip_serializing_if = "ExtensionsConfig::is_empty")]
    pub extensions: ExtensionsConfig,
}

/// Response type for the `sync` endpoint.
#[response(error = crate::Error)]
pub struct Response {
    /// Whether this response describes an initial sync (i.e. after the `pos` token has been
    /// discard by the server?).
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    pub initial: bool,

    /// Matches the `txn_id` sent by the request. Please see [`Request::txn_id`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub txn_id: Option<String>,

    /// The token to supply in the `pos` param of the next `/sync` request.
    pub pos: String,

    /// Updates on the order of rooms, mapped by the names we asked for.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub lists: BTreeMap<String, SyncList>,

    /// The updates on rooms.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub rooms: BTreeMap<OwnedRoomId, SlidingSyncRoom>,

    /// Extensions API.
    #[serde(default, skip_serializing_if = "Extensions::is_empty")]
    pub extensions: Extensions,

    /// The delta token to store for session recovery.
    ///
    /// The delta token is a future bandwidth optimisation to resume from an
    /// earlier session. If you received a delta token in your last response
    /// you can persist and it when establishing a new sessions to "resume"
    /// from the last state and not resend information you had stored. If you
    /// send a delta token, the server expects you to have stored the last
    /// state, if there is no delta token present the server will resend all
    /// information necessary to calculate the state.
    ///
    /// Please consult ["Bandwidth optimisations for persistent clients" of the MSC][MSC]
    /// for further details, expectations of the implementation and limitations
    /// to consider before implementing this.
    ///
    /// [MSC]: https://github.com/matrix-org/matrix-spec-proposals/blob/kegan/sync-v3/proposals/3575-sync.md#bandwidth-optimisations-for-persistent-clients
    pub delta_token: Option<String>,
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Default::default()
    }
}

impl Response {
    /// Creates a new `Response` with the given pos.
    pub fn new(pos: String) -> Self {
        Self {
            initial: Default::default(),
            txn_id: None,
            pos,
            delta_token: Default::default(),
            lists: Default::default(),
            rooms: Default::default(),
            extensions: Default::default(),
        }
    }
}

/// Filter for a sliding sync list, set at request.
///
/// All fields are applied with AND operators, hence if `is_dm`  is `true` and `is_encrypted` is
/// `true` then only encrypted DM rooms will be returned. The absence of fields implies no filter
/// on that criteria: it does NOT imply `false`.
///
/// Filters are considered _sticky_, meaning that the filter only has to be provided once and their
/// parameters 'sticks' for future requests until a new filter overwrites them.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SyncRequestListFilters {
    /// Whether to return DMs, non-DM rooms or both.
    ///
    /// Flag which only returns rooms present (or not) in the DM section of account data.
    /// If unset, both DM rooms and non-DM rooms are returned. If false, only non-DM rooms
    /// are returned. If true, only DM rooms are returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_dm: Option<bool>,

    /// Only list rooms that are spaces of these or all.
    ///
    /// A list of spaces which target rooms must be a part of. For every invited/joined
    /// room for this user, ensure that there is a parent space event which is in this list. If
    /// unset, all rooms are included. Servers MUST NOT navigate subspaces. It is up to the
    /// client to give a complete list of spaces to navigate. Only rooms directly in these
    /// spaces will be returned.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub spaces: Vec<String>,

    /// Whether to return encrypted, non-encrypted rooms or both.
    ///
    /// Flag which only returns rooms which have an `m.room.encryption` state event. If
    /// unset, both encrypted and unencrypted rooms are returned. If false, only unencrypted
    /// rooms are returned. If true, only encrypted rooms are returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_encrypted: Option<bool>,

    /// Whether to return invited Rooms, only joined rooms or both.
    ///
    /// Flag which only returns rooms the user is currently invited to. If unset, both
    /// invited and joined rooms are returned. If false, no invited rooms are returned. If
    /// true, only invited rooms are returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_invite: Option<bool>,

    /// Whether to return Rooms with tombstones, only rooms without tombstones or both.
    ///
    /// Flag which only returns rooms which have an `m.room.tombstone` state event. If unset,
    /// both tombstoned and un-tombstoned rooms are returned. If false, only un-tombstoned rooms
    /// are returned. If true, only tombstoned rooms are returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_tombstoned: Option<bool>,

    /// Only list rooms of given create-types or all.
    ///
    /// If specified, only rooms where the `m.room.create` event has a `type` matching one
    /// of the strings in this array will be returned. If this field is unset, all rooms are
    /// returned regardless of type. This can be used to get the initial set of spaces for an
    /// account.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub room_types: Vec<String>,

    /// Only list rooms that are not of these create-types, or all.
    ///
    /// Same as "room_types" but inverted. This can be used to filter out spaces from the room
    /// list.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub not_room_types: Vec<String>,

    /// Only list rooms matching the given string, or all.
    ///
    /// Filter the room name. Case-insensitive partial matching e.g 'foo' matches 'abFooab'.
    /// The term 'like' is inspired by SQL 'LIKE', and the text here is similar to '%foo%'.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_name_like: Option<String>,

    /// Filter the room based on its room tags.
    ///
    /// If multiple tags are present, a room can have
    /// any one of the listed tags (OR'd).
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub tags: Vec<String>,

    /// Filter the room based on its room tags.
    ///
    /// Takes priority over `tags`. For example, a room
    /// with tags A and B with filters `tags:[A]` `not_tags:[B]` would NOT be included because
    /// `not_tags` takes priority over `tags`. This filter is useful if your Rooms list does
    /// NOT include the list of favourite rooms again.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub not_tags: Vec<String>,

    /// Extensions may add further fields to the filters.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Sliding Sync Request for each list.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SyncRequestList {
    /// Put this list into the all-rooms-mode.
    ///
    /// Settings this to true will inform the server that, no matter how slow
    /// that might be, the clients wants all rooms the filters apply to. When operating
    /// in this mode, `ranges` and  `sort` will be ignored  there will be no movement operations
    /// (`DELETE` followed by `INSERT`) as the client has the entire list and can work out whatever
    /// sort order they wish. There will still be `DELETE` and `INSERT` operations when rooms are
    /// left or joined respectively. In addition, there will be an initial `SYNC` operation to let
    /// the client know which rooms in the rooms object were from this list.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    pub slow_get_all_rooms: bool,

    /// The ranges of rooms we're interested in.
    pub ranges: Vec<(UInt, UInt)>,

    /// The sort ordering applied to this list of rooms. Sticky.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sort: Vec<String>,

    /// The details to be included per room
    #[serde(flatten)]
    pub room_details: RoomDetailsConfig,

    /// If tombstoned rooms should be returned and if so, with what information attached.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_old_rooms: Option<IncludeOldRooms>,

    /// Request a stripped variant of membership events for the users used to calculate the room
    /// name.
    ///
    /// Sticky.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_heroes: Option<bool>,

    /// Filters to apply to the list before sorting. Sticky.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<SyncRequestListFilters>,

    /// An allow-list of event types which should be considered recent activity when sorting
    /// `by_recency`. By omitting event types from this field, clients can ensure that
    /// uninteresting events (e.g. a profil rename) do not cause a room to jump to the top of its
    /// list(s). Empty or omitted `bump_event_types` have no effect; all events in a room will be
    /// considered recent activity.
    ///
    /// NB. Changes to bump_event_types will NOT cause the room list to be reordered;
    /// it will only affect the ordering of rooms due to future updates.
    ///
    /// Sticky.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bump_event_types: Vec<TimelineEventType>,
}

/// Configuration for requesting room details.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RoomDetailsConfig {
    /// Required state for each room returned. An array of event type and state key tuples.
    ///
    /// Note that elements of this array are NOT sticky so they must be specified in full when they
    /// are changed. Sticky.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_state: Vec<(StateEventType, String)>,

    /// The maximum number of timeline events to return per room. Sticky.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeline_limit: Option<UInt>,
}

/// Configuration for old rooms to include
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct IncludeOldRooms {
    /// Required state for each room returned. An array of event type and state key tuples.
    ///
    /// Note that elements of this array are NOT sticky so they must be specified in full when they
    /// are changed. Sticky.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_state: Vec<(StateEventType, String)>,

    /// The maximum number of timeline events to return per room. Sticky.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeline_limit: Option<UInt>,
}

/// Configuration for room subscription
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RoomSubscription {
    /// Required state for each room returned. An array of event type and state key tuples.
    ///
    /// Note that elements of this array are NOT sticky so they must be specified in full when they
    /// are changed. Sticky.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_state: Vec<(StateEventType, String)>,

    /// The maximum number of timeline events to return per room. Sticky.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeline_limit: Option<UInt>,

    /// Include the room heroes. Sticky.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_heroes: Option<bool>,
}

/// Operation applied to the specific SlidingSyncList
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum SlidingOp {
    /// Full reset of the given window.
    Sync,
    /// Insert an item at the given point, moves all following entry by
    /// one to the next Empty or Invalid field.
    Insert,
    /// Drop this entry, moves all following entry up by one.
    Delete,
    /// Mark these as invaldiated.
    Invalidate,
}

/// Updates to joined rooms.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SyncList {
    /// The sync operation to apply, if any.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ops: Vec<SyncOp>,

    /// The total number of rooms found for this filter.
    pub count: UInt,
}

/// Updates to joined rooms.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SyncOp {
    /// The sync operation to apply.
    pub op: SlidingOp,

    /// The range this list update applies to.
    pub range: Option<(UInt, UInt)>,

    /// Or the specific index the update applies to.
    pub index: Option<UInt>,

    /// The list of room_ids updates to apply.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub room_ids: Vec<OwnedRoomId>,

    /// On insert and delete we are only receiving exactly one room_id.
    pub room_id: Option<OwnedRoomId>,
}

/// Updates to joined rooms.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SlidingSyncRoom {
    /// The name of the room as calculated by the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The avatar of the room.
    #[serde(default, skip_serializing_if = "JsOption::is_undefined")]
    pub avatar: JsOption<OwnedMxcUri>,

    /// Was this an initial response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial: Option<bool>,

    /// This is a direct message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_dm: Option<bool>,

    /// If this is `Some(_)`, this is a not-yet-accepted invite containing the given stripped state
    /// events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invite_state: Option<Vec<Raw<AnyStrippedStateEvent>>>,

    /// Counts of unread notifications for this room.
    #[serde(flatten, default, skip_serializing_if = "UnreadNotificationsCount::is_empty")]
    pub unread_notifications: UnreadNotificationsCount,

    /// The timeline of messages and state changes in the room.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub timeline: Vec<Raw<AnySyncTimelineEvent>>,

    /// Updates to the state at the beginning of the `timeline`.
    /// A list of state events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_state: Vec<Raw<AnySyncStateEvent>>,

    /// The prev_batch allowing you to paginate through the messages before the given ones.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_batch: Option<String>,

    /// True if the number of events returned was limited by the limit on the filter.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    pub limited: bool,

    /// The number of users with membership of `join`, including the client’s own user ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub joined_count: Option<UInt>,

    /// The number of users with membership of `invite`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invited_count: Option<UInt>,

    /// The number of timeline events which have just occurred and are not historical.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_live: Option<UInt>,

    /// The timestamp of the room.
    ///
    /// It's not to be confused with `origin_server_ts` of the latest event in the
    /// timeline. `bump_event_types` might "ignore” some events when computing the
    /// timestamp of the room. Thus, using this `timestamp` value is more accurate than
    /// relying on the latest event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<MilliSecondsSinceUnixEpoch>,

    /// Heroes of the room, if requested by a room subscription.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heroes: Option<Vec<SlidingSyncRoomHero>>,
}

impl SlidingSyncRoom {
    /// Creates an empty `Room`.
    pub fn new() -> Self {
        Default::default()
    }
}

/// A sliding sync room hero.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SlidingSyncRoomHero {
    /// The user ID of the hero.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<OwnedUserId>,

    /// The name of the hero.
    #[serde(rename = "displayname", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The avatar of the hero.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<OwnedMxcUri>,
}

/// Sliding-Sync extension configuration.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ExtensionsConfig {
    /// Request to devices messages with the given config.
    #[serde(default, skip_serializing_if = "ToDeviceConfig::is_empty")]
    pub to_device: ToDeviceConfig,

    /// Configure the end-to-end-encryption extension.
    #[serde(default, skip_serializing_if = "E2EEConfig::is_empty")]
    pub e2ee: E2EEConfig,

    /// Configure the account data extension.
    #[serde(default, skip_serializing_if = "AccountDataConfig::is_empty")]
    pub account_data: AccountDataConfig,

    /// Request to receipt information with the given config.
    #[serde(default, skip_serializing_if = "ReceiptsConfig::is_empty")]
    pub receipts: ReceiptsConfig,

    /// Request to typing information with the given config.
    #[serde(default, skip_serializing_if = "TypingConfig::is_empty")]
    pub typing: TypingConfig,

    /// Extensions may add further fields to the list.
    #[serde(flatten)]
    other: BTreeMap<String, serde_json::Value>,
}

impl ExtensionsConfig {
    /// Whether all fields are empty or `None`.
    pub fn is_empty(&self) -> bool {
        self.to_device.is_empty()
            && self.e2ee.is_empty()
            && self.account_data.is_empty()
            && self.receipts.is_empty()
            && self.typing.is_empty()
            && self.other.is_empty()
    }
}

/// Extensions specific response data.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Extensions {
    /// To-device extension in response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_device: Option<ToDevice>,

    /// E2EE extension in response.
    #[serde(default, skip_serializing_if = "E2EE::is_empty")]
    pub e2ee: E2EE,

    /// Account data extension in response.
    #[serde(default, skip_serializing_if = "AccountData::is_empty")]
    pub account_data: AccountData,

    /// Receipt data extension in response.
    #[serde(default, skip_serializing_if = "Receipts::is_empty")]
    pub receipts: Receipts,

    /// Typing data extension in response.
    #[serde(default, skip_serializing_if = "Typing::is_empty")]
    pub typing: Typing,
}

impl Extensions {
    /// Whether the extension data is empty.
    ///
    /// True if neither to-device, e2ee nor account data are to be found.
    pub fn is_empty(&self) -> bool {
        self.to_device.is_none()
            && self.e2ee.is_empty()
            && self.account_data.is_empty()
            && self.receipts.is_empty()
            && self.typing.is_empty()
    }
}

/// To-device messages extension configuration.
///
/// According to [MSC3885](https://github.com/matrix-org/matrix-spec-proposals/pull/3885).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ToDeviceConfig {
    /// Activate or deactivate this extension. Sticky.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Max number of to-device messages per response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<UInt>,

    /// Give messages since this token only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,

    /// List of list names for which to-device events should be enabled.
    ///
    /// If not defined, will be enabled for *all* the lists appearing in the request.
    /// If defined and empty, will be disabled for all the lists.
    ///
    /// Sticky.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lists: Option<Vec<String>>,

    /// List of room names for which to-device events should be enabled.
    ///
    /// If not defined, will be enabled for *all* the rooms appearing in the `room_subscriptions`.
    /// If defined and empty, will be disabled for all the rooms.
    ///
    /// Sticky.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rooms: Option<Vec<OwnedRoomId>>,
}

impl ToDeviceConfig {
    /// Whether all fields are empty or `None`.
    pub fn is_empty(&self) -> bool {
        self.enabled.is_none() && self.limit.is_none() && self.since.is_none()
    }
}

/// To-device messages extension response.
///
/// According to [MSC3885](https://github.com/matrix-org/matrix-spec-proposals/pull/3885).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ToDevice {
    /// Fetch the next batch from this entry.
    pub next_batch: String,

    /// The to-device Events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Raw<AnyToDeviceEvent>>,
}

/// E2EE extension configuration.
///
/// According to [MSC3884](https://github.com/matrix-org/matrix-spec-proposals/pull/3884).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct E2EEConfig {
    /// Activate or deactivate this extension. Sticky.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

impl E2EEConfig {
    /// Whether all fields are empty or `None`.
    pub fn is_empty(&self) -> bool {
        self.enabled.is_none()
    }
}

/// E2EE extension response data.
///
/// According to [MSC3884](https://github.com/matrix-org/matrix-spec-proposals/pull/3884).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct E2EE {
    /// Information on E2EE device updates.
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_unused_fallback_key_types: Option<Vec<DeviceKeyAlgorithm>>,
}

impl E2EE {
    /// Whether all fields are empty or `None`.
    pub fn is_empty(&self) -> bool {
        self.device_lists.is_empty()
            && self.device_one_time_keys_count.is_empty()
            && self.device_unused_fallback_key_types.is_none()
    }
}

/// Account-data extension configuration.
///
/// Not yet part of the spec proposal. Taken from the reference implementation
/// <https://github.com/matrix-org/sliding-sync/blob/main/sync3/extensions/account_data.go>
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct AccountDataConfig {
    /// Activate or deactivate this extension. Sticky.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// List of list names for which account data should be enabled.
    ///
    /// This is specific to room account data (e.g. user-defined room tags).
    ///
    /// If not defined, will be enabled for *all* the lists appearing in the request.
    /// If defined and empty, will be disabled for all the lists.
    ///
    /// Sticky.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lists: Option<Vec<String>>,

    /// List of room names for which account data should be enabled.
    ///
    /// This is specific to room account data (e.g. user-defined room tags).
    ///
    /// If not defined, will be enabled for *all* the rooms appearing in the `room_subscriptions`.
    /// If defined and empty, will be disabled for all the rooms.
    ///
    /// Sticky.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rooms: Option<Vec<OwnedRoomId>>,
}

impl AccountDataConfig {
    /// Whether all fields are empty or `None`.
    pub fn is_empty(&self) -> bool {
        self.enabled.is_none()
    }
}

/// Account-data extension response data.
///
/// Not yet part of the spec proposal. Taken from the reference implementation
/// <https://github.com/matrix-org/sliding-sync/blob/main/sync3/extensions/account_data.go>
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct AccountData {
    /// The global private data created by this user.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub global: Vec<Raw<AnyGlobalAccountDataEvent>>,

    /// The private data that this user has attached to each room.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub rooms: BTreeMap<OwnedRoomId, Vec<Raw<AnyRoomAccountDataEvent>>>,
}

impl AccountData {
    /// Whether all fields are empty or `None`.
    pub fn is_empty(&self) -> bool {
        self.global.is_empty() && self.rooms.is_empty()
    }
}

/// Single entry for a room-related read receipt configuration in `ReceiptsConfig`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum RoomReceiptConfig {
    /// Get read receipts for all the subscribed rooms.
    AllSubscribed,
    /// Get read receipts for this particular room.
    Room(OwnedRoomId),
}

impl Serialize for RoomReceiptConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            RoomReceiptConfig::AllSubscribed => serializer.serialize_str("*"),
            RoomReceiptConfig::Room(r) => r.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for RoomReceiptConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        match deserialize_cow_str(deserializer)?.as_ref() {
            "*" => Ok(RoomReceiptConfig::AllSubscribed),
            other => Ok(RoomReceiptConfig::Room(
                RoomId::parse(other).map_err(D::Error::custom)?.to_owned(),
            )),
        }
    }
}

/// Receipt extension configuration.
///
/// According to [MSC3960](https://github.com/matrix-org/matrix-spec-proposals/pull/3960)
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ReceiptsConfig {
    /// Activate or deactivate this extension. Sticky.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// List of list names for which receipts should be enabled.
    ///
    /// If not defined, will be enabled for *all* the lists appearing in the request.
    /// If defined and empty, will be disabled for all the lists.
    ///
    /// Sticky.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lists: Option<Vec<String>>,

    /// List of room names for which receipts should be enabled.
    ///
    /// If not defined, will be enabled for *all* the rooms appearing in the `room_subscriptions`.
    /// If defined and empty, will be disabled for all the rooms.
    ///
    /// Sticky.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rooms: Option<Vec<RoomReceiptConfig>>,
}

impl ReceiptsConfig {
    /// Whether all fields are empty or `None`.
    pub fn is_empty(&self) -> bool {
        self.enabled.is_none()
    }
}

/// Receipt extension response data.
///
/// According to [MSC3960](https://github.com/matrix-org/matrix-spec-proposals/pull/3960)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Receipts {
    /// The ephemeral receipt room event for each room
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub rooms: BTreeMap<OwnedRoomId, Raw<SyncReceiptEvent>>,
}

impl Receipts {
    /// Whether all fields are empty or `None`.
    pub fn is_empty(&self) -> bool {
        self.rooms.is_empty()
    }
}

/// Typing extension configuration.
///
/// Not yet part of the spec proposal. Taken from the reference implementation
/// <https://github.com/matrix-org/sliding-sync/blob/main/sync3/extensions/typing.go>
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct TypingConfig {
    /// Activate or deactivate this extension. Sticky.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// List of list names for which typing notifications should be enabled.
    ///
    /// If not defined, will be enabled for *all* the lists appearing in the request.
    /// If defined and empty, will be disabled for all the lists.
    ///
    /// Sticky.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lists: Option<Vec<String>>,

    /// List of room names for which typing notifications should be enabled.
    ///
    /// If not defined, will be enabled for *all* the rooms appearing in the `room_subscriptions`.
    /// If defined and empty, will be disabled for all the rooms.
    ///
    /// Sticky.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rooms: Option<Vec<OwnedRoomId>>,
}

impl TypingConfig {
    /// Whether all fields are empty or `None`.
    pub fn is_empty(&self) -> bool {
        self.enabled.is_none()
    }
}

/// Typing extension response data.
///
/// Not yet part of the spec proposal. Taken from the reference implementation
/// <https://github.com/matrix-org/sliding-sync/blob/main/sync3/extensions/typing.go>
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Typing {
    /// The ephemeral typing event for each room
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub rooms: BTreeMap<OwnedRoomId, Raw<SyncTypingEvent>>,
}

impl Typing {
    /// Whether all fields are empty or `None`.
    pub fn is_empty(&self) -> bool {
        self.rooms.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use ruma_common::owned_room_id;

    use crate::sync::sync_events::v4::RoomReceiptConfig;

    #[test]
    fn serialize_room_receipt_config() {
        let entry = RoomReceiptConfig::AllSubscribed;
        assert_eq!(serde_json::to_string(&entry).unwrap().as_str(), r#""*""#);

        let entry = RoomReceiptConfig::Room(owned_room_id!("!n8f893n9:example.com"));
        assert_eq!(serde_json::to_string(&entry).unwrap().as_str(), r#""!n8f893n9:example.com""#);
    }

    #[test]
    fn deserialize_room_receipt_config() {
        assert_eq!(
            serde_json::from_str::<RoomReceiptConfig>(r#""*""#).unwrap(),
            RoomReceiptConfig::AllSubscribed
        );

        assert_eq!(
            serde_json::from_str::<RoomReceiptConfig>(r#""!n8f893n9:example.com""#).unwrap(),
            RoomReceiptConfig::Room(owned_room_id!("!n8f893n9:example.com"))
        );
    }
}
