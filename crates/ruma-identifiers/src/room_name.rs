//! Matrix room name.

use ruma_identifiers_validation::room_name::validate;

opaque_identifier_validated! {
    /// The name of a room.
    ///
    /// It can't exceed 255 bytes or be empty.
    pub type RoomName [ validate ];
}
