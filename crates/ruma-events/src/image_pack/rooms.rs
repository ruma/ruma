//! Types for the [`m.image_pack.rooms`] account data.
//!
//! [`m.image_pack.rooms`]: https://spec.matrix.org/v1.19/client-server-api/#mimage_packrooms

use std::collections::BTreeMap;

use ruma_common::OwnedRoomId;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of an [`m.image_pack.rooms`] event.
///
/// [`m.image_pack.rooms`]: https://spec.matrix.org/v1.19/client-server-api/#mimage_packrooms
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.image_pack.rooms", kind = GlobalAccountData)]
pub struct ImagePackRoomsEventContent {
    /// A map of room ID to a map of state key to an empty object.
    ///
    /// Each entry references a specific [`m.room.image_pack`] state event that the user has
    /// enabled globally.
    ///
    /// [`m.room.image_pack`]: https://spec.matrix.org/v1.19/client-server-api/#mroomimage_pack
    pub rooms: BTreeMap<OwnedRoomId, BTreeMap<String, RoomImagePackMeta>>,
}

impl ImagePackRoomsEventContent {
    /// Creates a new `ImagePackRoomsEventContent` with the given map of enabled image packs in each
    /// room.
    pub fn new(rooms: BTreeMap<OwnedRoomId, BTreeMap<String, RoomImagePackMeta>>) -> Self {
        Self { rooms }
    }
}

/// Additional metadata for a globally enabled room image pack.
///
/// This is currently empty.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RoomImagePackMeta {}

impl RoomImagePackMeta {
    /// Creates a new empty `RoomImagePackMeta`.
    pub fn new() -> Self {
        Self {}
    }
}
