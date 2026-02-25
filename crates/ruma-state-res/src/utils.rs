use ruma_common::{EventId, IdParseError, RoomId};

/// Convenience extension trait for [`RoomId`].
pub(crate) trait RoomIdExt {
    /// Get the event ID of the `m.room.create` event of the room from a PDU, for room versions
    /// using it as the room ID.
    fn room_create_event_id(&self) -> Result<EventId, IdParseError>;
}

impl RoomIdExt for RoomId {
    fn room_create_event_id(&self) -> Result<EventId, IdParseError> {
        EventId::parse(format!("${}", self.strip_sigil()))
    }
}
