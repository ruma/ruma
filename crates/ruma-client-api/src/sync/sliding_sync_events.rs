//! [POST /_matrix/client/unstable/org.matrix.msc3575/sync](https://github.com/matrix-org/matrix-doc/blob/kegan/sync-v3/proposals/3575-sync.md)

use std::{collections::BTreeMap, time::Duration};

use super::sync_events::v3::UnreadNotificationsCount;
use js_int::UInt;
use ruma_common::{
    api::ruma_api,
    events::{AnyStrippedStateEvent, AnySyncRoomEvent, AnySyncStateEvent, RoomEventType},
    serde::{duration::opt_ms, Raw},
    OwnedRoomId,
};
use serde::{Deserialize, Serialize};

/// Filter for a sliding sync list, set at request.
///
/// All fields are applied with AND operators, hence if is_dm:true and is_encrypted:true
/// then only Encrypted DM rooms will be returned. The absence of fields implies no filter
/// on that criteria: it does NOT imply 'false'.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SyncRequestListFilters {
    // These fields may be expanded through use of extensions.
    /// Sticky. Flag which only returns rooms present (or not) in the DM section of account data.
    /// If unset, both DM rooms and non-DM rooms are returned. If false, only non-DM rooms
    /// are returned. If true, only DM rooms are returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_dm: Option<bool>,

    /// Sticky. A list of spaces which target rooms must be a part of. For every invited/joined
    /// room for this user, ensure that there is a parent space event which is in this list. If
    /// unset, all rooms are included. Servers MUST NOT navigate subspaces. It is up to the
    /// client to give a complete list of spaces to navigate. Only rooms directly in these
    /// spaces will be returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spaces: Option<Vec<String>>,

    /// Sticky. Flag which only returns rooms which have an `m.room.encryption` state event. If
    /// unset, both encrypted and unencrypted rooms are returned. If false, only unencrypted
    /// rooms are returned. If true, only encrypted rooms are returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_encrypted: Option<bool>,

    /// Sticky. Flag which only returns rooms the user is currently invited to. If unset, both
    /// invited and joined rooms are returned. If false, no invited rooms are returned. If
    /// true, only invited rooms are returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_invite: Option<bool>,

    /// Flag which only returns rooms which have an `m.room.tombstone` state event. If unset,
    /// both tombstoned and un-tombstoned rooms are returned. If false, only un-tombstoned rooms
    /// are returned. If true, only tombstoned rooms are returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_tombstoned: Option<bool>,

    /// If specified, only rooms where the `m.room.create` event has a `type` matching one
    /// of the strings in this array will be returned. If this field is unset, all rooms are
    /// returned regardless of type. This can be used to get the initial set of spaces for an
    /// account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_types: Option<Vec<String>>,

    /// Same as "room_types" but inverted. This can be used to filter out spaces from the room
    /// list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_room_types: Option<Vec<String>>,

    /// Filter the room name. Case-insensitive partial matching e.g 'foo' matches 'abFooab'.
    /// The term 'like' is inspired by SQL 'LIKE', and the text here is similar to '%foo%'.
    pub room_name_like: Option<String>,
}

/// Sliding Sync Request for each list
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SyncRequestList {
    /// The ranges of rooms we're interested in
    pub ranges: Vec<(UInt, UInt)>,

    /// Sticky. The sort ordering applied to this list of rooms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<Vec<String>>,

    /// Sticky. Required state for each room returned. An array of event type and state key tuples.
    /// Note that elements of this array are NOT sticky so they must be specified in full when they
    /// are changed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_state: Option<Vec<(RoomEventType, String)>>,

    /// Sticky. The maximum number of timeline events to return per room
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeline_limit: Option<UInt>,

    /// Sticky. Filters to apply to the list before sorting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<Raw<SyncRequestListFilters>>,
}

/// The RoomSubscriptions of the SlidingSync Request
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RoomSubscription {
    /// Sticky. Required state for each room returned. An array of event type and state key tuples.
    /// Note that elements of this array are NOT sticky so they must be specified in full when they
    /// are changed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_state: Option<Vec<(RoomEventType, String)>>,

    /// Sticky. The maximum number of timeline events to return per room
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeline_limit: Option<UInt>,
}

/// Sliding Sync Request
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SyncRequest {
    /// The lists of rooms we're interested in
    pub lists: Vec<Raw<SyncRequestList>>,

    /// Specific rooms and event types that we want to receive events from
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_subscriptions: Option<BTreeMap<OwnedRoomId, RoomSubscription>>,

    /// Specific rooms we no longer want to receive events from
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unsubscribe_rooms: Option<Vec<OwnedRoomId>>,
}

ruma_api! {
    metadata: {
        description: "Get all new events in a sliding window of rooms since the last sync or a given point of time.",
        method: POST,
        name: "sync",
        added: 1.0,
        stable_path: "/_matrix/client/v3/sync",
        // unstable_path: "/_matrix/client/unstable/org.matrix.msc3575/sync",
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

        /// The maximum time to poll in milliseconds before returning this request.
        #[serde(
            with = "opt_ms",
            default,
            skip_serializing_if = "Option::is_none",
        )]
        #[ruma_api(query)]
        pub timeout: Option<Duration>,

        /// The sync request body to send
        #[ruma_api(body)]
        pub body: SyncRequest,

    }

    response: {
        /// Present and true if this response describes an initial sync
        /// (i.e. after the `pos` token has been discard by the server?)
        #[serde(skip_serializing_if = "Option::is_none")]
        pub initial: Option<bool>,

        /// The token to supply in the `pos` param of the next `/sync` request.
        pub pos: String,

        /// Updates to the sliding room list
        #[serde()]
        pub lists: Vec<SyncList>,

        /// The updates on rooms
        pub rooms: Option<BTreeMap<OwnedRoomId, SlidingSyncRoom>>
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

/// Operation applied to the specific SlidingSyncList
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SlidingOp {
    /// Full reset of the given window
    Sync,
    /// Insert an item at the given point, moves all following entry by
    /// one to the next Empty or Invalid field
    Insert,
    /// Drop this entry, moves all following entry up by one
    Delete,
    /// These have changed
    Update,
    /// Mark these as invaldiated
    Invalidate,
}

/// Updates to joined rooms.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SyncList {
    /// The sync operation to apply, if any
    pub ops: Option<Vec<SyncOp>>,

    /// The total number of rooms found for this filter
    pub count: UInt,
}

/// Updates to joined rooms.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SyncOp {
    /// The sync operation to apply
    pub op: SlidingOp,

    /// The range this list update applies to
    pub range: Option<(UInt, UInt)>,

    /// Or the specific index the update applies to
    pub index: Option<UInt>,

    /// The list of room_ids updates to apply
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_ids: Option<Vec<OwnedRoomId>>,

    /// On insert we are only receiving exactly one room_id
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_id: Option<OwnedRoomId>,
}

/// Updates to joined rooms.
/// XXX: it'd be much nicer if we could use the same object to describe
/// both updates in room subscriptions as well as ops.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SlidingSyncRoom {
    /// The name of the room as calculated by the server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Was this an initial response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial: Option<bool>,

    /// This is a direct message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_dm: Option<bool>,

    /// This is not-yet-accepted invite, with the following sync state events
    /// the room must be considered in invite state as long as the Option is not None
    /// even if there are no state events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invite_state: Option<Vec<Raw<AnyStrippedStateEvent>>>,

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

    /// The prev_batch allowing you to paginate through the messages before the given ones
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_batch: Option<String>,
}

impl SlidingSyncRoom {
    /// Creates an empty `Room`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns true if there are no updates in the room.
    pub fn is_empty(&self) -> bool {
        self.unread_notifications.is_empty()
            && self.timeline.is_empty()
            && self.required_state.is_empty()
    }
}
