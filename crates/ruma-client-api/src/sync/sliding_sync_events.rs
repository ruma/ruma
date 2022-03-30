//! [POST /_matrix/client/unstable/org.matrix.msc3575/sync](https://github.com/matrix-org/matrix-doc/blob/kegan/sync-v3/proposals/3575-sync.md)

use std::{collections::BTreeMap, time::Duration};

use js_int::UInt;
use ruma_common::{
    api::ruma_api,
    presence::PresenceState,
    events::{
        presence::PresenceEvent, AnySyncRoomEvent, AnySyncStateEvent,
        EventType
    },
    DeviceKeyAlgorithm, RoomId, UserId,
    serde::Raw, serde::duration::opt_ms,
};
use serde::{Deserialize, Serialize};

use crate::filter::{FilterDefinition};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SyncRequestListFilters {
    // All fields are applied with AND operators, hence if is_dm:true and is_encrypted:true
    // then only Encrypted DM rooms will be returned. The absence of fields implies no filter
    // on that criteria: it does NOT imply 'false'.
    // These fields may be expanded through use of extensions.

    /// Sticky. Flag which only returns rooms present (or not) in the DM section of account data.
    /// If unset, both DM rooms and non-DM rooms are returned. If false, only non-DM rooms
    /// are returned. If true, only DM rooms are returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_dm: Option<bool>,

    /// Sticky. A list of spaces which target rooms must be a part of. For every invited/joined room for
    /// this user, ensure that there is a parent space event which is in this list. If unset, all
    /// rooms are included. Servers MUST NOT navigate subspaces. It is up to the client to
    /// give a complete list of spaces to navigate. Only rooms directly in these spaces will be
    /// returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spaces: Option<Vec<String>>,

    /// Sticky. Flag which only returns rooms which have an `m.room.encryption` state event. If unset,
    /// both encrypted and unencrypted rooms are returned. If false, only unencrypted rooms
    /// are returned. If true, only encrypted rooms are returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_encrypted: Option<bool>,

    /// Sticky. Flag which only returns rooms the user is currently invited to. If unset, both invited
    /// and joined rooms are returned. If false, no invited rooms are returned. If true, only
    /// invited rooms are returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_invite: Option<bool>,
}

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
    pub required_state: Option<Vec<(EventType, String)>>,

    /// Sticky. The maximum number of timeline events to return per room
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeline_limit: Option<UInt>,

    /// Sticky. Filters to apply to the list before sorting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<Raw<SyncRequestListFilters>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RoomSubscription {
    /// Sticky. Required state for each room returned. An array of event type and state key tuples.
    /// Note that elements of this array are NOT sticky so they must be specified in full when they
    /// are changed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_state: Option<Vec<(EventType, String)>>,

    /// Sticky. The maximum number of timeline events to return per room
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeline_limit: Option<UInt>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SyncRequest {
    /// The lists of rooms we're interested in
    pub lists: Vec<Raw<SyncRequestList>>,

    /// Specific rooms and event types that we want to receive events from
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_subscriptions: Option<BTreeMap<Box<RoomId>, RoomSubscription>>,

    /// Specific rooms we no longer want to receive events from
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unsubscribe_rooms: Option<Vec<Box<RoomId>>>,
}

ruma_api! {
    metadata: {
        description: "Get all new events in a sliding window of rooms since the last sync or a given point of time.",
        method: POST,
        name: "sync",
        added: 1.0,
        stable_path: "/_matrix/client/v3/sync",
        //unstable_path: "/_matrix/client/unstable/org.matrix.msc3575/sync",
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
        #[serde(skip_serializing_if = "Option::is_none")]
        pub ops: Option<Vec<Raw<SyncOp>>>,

        /// The number of available rooms(?)
        pub counts: Vec<UInt>,

        /// Updates to subscribed rooms.
        pub room_subscriptions: Option<BTreeMap<Box<RoomId>, Room>>,
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
            ops: Default::default(),
            counts: Default::default(),
            room_subscriptions: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all="UPPERCASE")]
pub enum SlidingOp {
    Sync,
    Insert,
    Delete,
    Update,
    Invalidate,
}

/// Updates to joined rooms.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SyncOp {
    /// Which room list this op refers to
    pub list: UInt,

    /// The range this list update applies to
    pub range: (UInt, UInt),

    /// The sync operation to apply
    pub op: SlidingOp,

    /// The list of room updates to apply
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rooms: Option<Vec<Room>>,

    /// On insert we are only receiving exactly one room
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room: Option<Room>,
}

/// Updates to joined rooms.
/// XXX: it'd be much nicer if we could use the same object to describe
/// both updates in room subscriptions as well as ops.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Room {
    /// The room ID (only set on op updates due to the API being asymmetric)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_id: Option<String>,

    /// The name of the room as calculated by the server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Was this an initial response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial: Option<bool>,

    /// The number of unread notifications for this room with the highlight flag set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight_count: Option<UInt>,

    /// The total number of unread notifications for this room.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_count: Option<UInt>,

    /// The timeline of messages and state changes in the room.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub timeline: Vec<Raw<AnySyncRoomEvent>>,

    /// Updates to the state at the beginning of the `timeline`.
    /// A list of state events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_state: Vec<Raw<AnySyncStateEvent>>,
}

impl Room {
    /// Creates an empty `Room`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns true if there are no updates in the room.
    pub fn is_empty(&self) -> bool {
        self.highlight_count.is_none()
            && self.notification_count.is_none()
            && self.timeline.is_empty()
            && self.required_state.is_empty()
    }
}
