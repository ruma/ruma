use ruma::identifiers::{EventId, RoomId};

use crate::StateEvent;

pub trait StateStore {
    /// Return a single event based on the EventId.
    fn get_event(&self, event_id: &EventId) -> Result<StateEvent, String>;

    /// Returns the events that correspond to the `event_ids` sorted in the same order.
    fn get_events(&self, event_ids: &[EventId]) -> Result<Vec<StateEvent>, String>;

    /// Returns a Vec of the related auth events to the given `event`.
    fn auth_event_ids(
        &self,
        room_id: &RoomId,
        event_ids: &[EventId],
    ) -> Result<Vec<EventId>, String>;

    /// Returns a Vec<EventId> representing the difference in auth chains of the given `events`.
    fn auth_chain_diff(
        &self,
        room_id: &RoomId,
        event_id: Vec<Vec<EventId>>,
    ) -> Result<Vec<EventId>, String>;
}
