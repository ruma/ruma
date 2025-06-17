use ruma_common::{EventId, IdParseError, OwnedEventId, RoomId};

/// Convenience extension trait for [`RoomId`].
pub(crate) trait RoomIdExt {
    /// Get the event ID of the `m.room.create` event of the room from a PDU, for room versions
    /// using it as the room ID.
    fn room_create_event_id(&self) -> Result<OwnedEventId, IdParseError>;
}

impl<T> RoomIdExt for T
where
    T: AsRef<RoomId>,
{
    fn room_create_event_id(&self) -> Result<OwnedEventId, IdParseError> {
        EventId::parse(format!("${}", self.as_ref().strip_sigil()))
    }
}
