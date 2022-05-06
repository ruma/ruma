//! Matrix room name.

use ruma_macros::IdZst;

/// The name of a room.
///
/// It can't exceed 255 bytes or be empty.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, IdZst)]
#[ruma_id(validate = ruma_identifiers_validation::room_name::validate)]
pub struct RoomName(str);
