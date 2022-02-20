//! Matrix room name.

/// The name of a room.
///
/// It can't exceed 255 bytes or be empty.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoomName(str);

owned_identifier!(OwnedRoomName, RoomName);

opaque_identifier_validated!(
    RoomName,
    OwnedRoomName,
    ruma_identifiers_validation::room_name::validate
);
