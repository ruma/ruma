//! [POST /_matrix/client/unstable/org.matrix.msc3575/sync](https://github.com/matrix-org/matrix-doc/blob/kegan/sync-v3/proposals/3575-sync.md)

use std::{collections::BTreeMap, time::Duration};

use super::UnreadNotificationsCount;
use js_int::UInt;
use ruma_common::{
    api::ruma_api,
    events::{AnyStrippedStateEvent, AnySyncRoomEvent, AnySyncStateEvent, RoomEventType},
    serde::{duration::opt_ms, Raw},
    OwnedRoomId,
};
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata: {
        description: "Get all new events in a sliding window of rooms since the last sync or a given point of time.",
        method: POST,
        name: "sync",
        // added: 1.4,
        // stable_path: "/_matrix/client/v4/sync",
        unstable_path: "/_matrix/client/unstable/org.matrix.msc3575/sync",
        rate_limited: false,
        authentication: AccessToken,
    }

    #[derive(Default)]
    request: {
        /// A point in time to continue a sync from.
        ///
        /// Should be a token from the `pos` field of a previous `/sync`
        /// response.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub pos: Option<&'a str>,

        /// Allows clients to know what request params reached the server,
        /// functionally similar to txn IDs on /send for events.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub txn_id: Option<&'a str>,

        /// The maximum time to poll before responding to this request.
        #[serde(
            with = "opt_ms",
            default,
            skip_serializing_if = "Option::is_none",
        )]
        #[ruma_api(query)]
        pub timeout: Option<Duration>,

        /// The lists of rooms we're interested in.
        pub lists: &'a [SyncRequestList],

        /// Specific rooms and event types that we want to receive events from.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub room_subscriptions: BTreeMap<OwnedRoomId, RoomSubscription>,

        /// Specific rooms we no longer want to receive events from.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        pub unsubscribe_rooms: &'a [OwnedRoomId],

        /// Extensions API.
        #[serde(skip_serializing_if = "BTreeMap::is_empty")]
        pub extensions: BTreeMap<String, serde_json::Value>,
    }

    response: {
        /// Whether this response describes an initial sync (i.e. after the `pos` token has been
        /// discard by the server?).
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub initial: bool,

        /// The token to supply in the `pos` param of the next `/sync` request.
        pub pos: String,

        /// Updates to the sliding room list.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub lists: Vec<SyncList>,

        /// The updates on rooms.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub rooms: BTreeMap<OwnedRoomId, SlidingSyncRoom>,
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
    /// Creates a new `Response` with the given pos.
    pub fn new(pos: String) -> Self {
        Self {
            initial: Default::default(),
            pos,
            lists: Default::default(),
            rooms: Default::default(),
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
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
    pub room_name_like: Option<String>,

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

    /// Required state for each room returned. An array of event type and state key tuples.
    /// Note that elements of this array are NOT sticky so they must be specified in full when they
    /// are changed. Sticky.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_state: Vec<(RoomEventType, String)>,

    /// The maximum number of timeline events to return per room. Sticky.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeline_limit: Option<UInt>,

    /// Filters to apply to the list before sorting. Sticky.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<SyncRequestListFilters>,
}

/// The RoomSubscriptions of the SlidingSync Request
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RoomSubscription {
    /// Required state for each room returned. An array of event type and state key tuples.
    ///
    /// Note that elements of this array are NOT sticky so they must be specified in full when they
    /// are changed. Sticky.

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_state: Vec<(RoomEventType, String)>,

    /// The maximum number of timeline events to return per room. Sticky.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeline_limit: Option<UInt>,
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

    /// Was this an initial response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial: Option<bool>,

    /// This is a direct message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_dm: Option<bool>,

    /// This is not-yet-accepted invite, with the following sync state events
    /// the room must be considered in invite state as long as the Option is not None
    /// even if there are no state events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invite_state: Vec<Raw<AnyStrippedStateEvent>>,

    /// Counts of unread notifications for this room.
    #[serde(flatten, default, skip_serializing_if = "UnreadNotificationsCount::is_empty")]
    pub unread_notifications: UnreadNotificationsCount,

    /// The timeline of messages and state changes in the room.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub timeline: Vec<Raw<AnySyncRoomEvent>>,

    /// Updates to the state at the beginning of the `timeline`.
    /// A list of state events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_state: Vec<Raw<AnySyncStateEvent>>,

    /// The prev_batch allowing you to paginate through the messages before the given ones.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_batch: Option<String>,
}

impl SlidingSyncRoom {
    /// Creates an empty `Room`.
    pub fn new() -> Self {
        Default::default()
    }
}
