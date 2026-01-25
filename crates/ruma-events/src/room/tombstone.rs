//! Types for the [`m.room.tombstone`] event.
//!
//! [`m.room.tombstone`]: https://spec.matrix.org/latest/client-server-api/#mroomtombstone

use ruma_common::OwnedRoomId;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{EmptyStateKey, PossiblyRedactedStateEventContent, StateEventType, StaticEventContent};

/// The content of an `m.room.tombstone` event.
///
/// A state event signifying that a room has been upgraded to a different room version, and that
/// clients should go there.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(
    type = "m.room.tombstone",
    kind = State,
    state_key_type = EmptyStateKey,
    custom_possibly_redacted,
)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RoomTombstoneEventContent {
    /// A server-defined message.
    ///
    /// If the `compat-optional` feature is enabled, this field being absent in JSON will result
    /// in an empty string instead of an error when deserializing.
    #[cfg_attr(feature = "compat-optional", serde(default))]
    pub body: String,

    /// The new room the client should be visiting.
    pub replacement_room: OwnedRoomId,
}

impl RoomTombstoneEventContent {
    /// Creates a new `RoomTombstoneEventContent` with the given body and replacement room ID.
    pub fn new(body: String, replacement_room: OwnedRoomId) -> Self {
        Self { body, replacement_room }
    }
}

/// The possibly redacted form of [`RoomTombstoneEventContent`].
///
/// This type is used when it's not obvious whether the content is redacted or not.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct PossiblyRedactedRoomTombstoneEventContent {
    /// A server-defined message.
    pub body: Option<String>,

    /// The new room the client should be visiting.
    pub replacement_room: Option<OwnedRoomId>,
}

impl PossiblyRedactedStateEventContent for PossiblyRedactedRoomTombstoneEventContent {
    type StateKey = EmptyStateKey;

    fn event_type(&self) -> StateEventType {
        StateEventType::RoomTombstone
    }
}

impl StaticEventContent for PossiblyRedactedRoomTombstoneEventContent {
    const TYPE: &'static str = RoomTombstoneEventContent::TYPE;
    type IsPrefix = <RoomTombstoneEventContent as StaticEventContent>::IsPrefix;
}

impl From<RoomTombstoneEventContent> for PossiblyRedactedRoomTombstoneEventContent {
    fn from(value: RoomTombstoneEventContent) -> Self {
        let RoomTombstoneEventContent { body, replacement_room } = value;
        Self { body: Some(body), replacement_room: Some(replacement_room) }
    }
}

impl From<RedactedRoomTombstoneEventContent> for PossiblyRedactedRoomTombstoneEventContent {
    fn from(_value: RedactedRoomTombstoneEventContent) -> Self {
        Self { body: None, replacement_room: None }
    }
}
