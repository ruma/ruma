#![allow(clippy::exhaustive_enums, clippy::exhaustive_structs)]

use ruma_common::{OwnedRoomId, owned_room_id, room_version_rules::RoomIdFormatVersion};

mod factory;
mod pdu;

pub use self::{factory::*, pdu::*};

/// Get the default room ID in the proper format according to the room version rules.
pub fn default_room_id(format: &RoomIdFormatVersion) -> OwnedRoomId {
    match format {
        RoomIdFormatVersion::V1 => owned_room_id!("!room:matrix.local"),
        // The default ID of the `m.room.create` event.
        RoomIdFormatVersion::V2 => owned_room_id!("!room-create"),
        _ => panic!("Unsupported RoomIdFormatVersion"),
    }
}
