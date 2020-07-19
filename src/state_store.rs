use ruma::identifiers::{EventId, RoomId, RoomVersionId};

use crate::StateEvent;

pub trait StateStore {
    /// Return a single event based on the EventId.
    fn get_event(&self, event_id: &EventId) -> Result<StateEvent, String>;

    /// Returns the events that correspond to the `event_ids` sorted in the same order.
    fn get_events(&self, event_ids: &[EventId]) -> Result<Vec<StateEvent>, String>;

    /// Returns a Vec of the related auth events to the given `event`.
    fn auth_event_ids(&self, room_id: &RoomId, event_id: &EventId) -> Result<Vec<EventId>, String>;

    /// Returns a Vec<EventId> representing the difference in auth chains of the given `events`.
    fn auth_chain_diff(&self, event_id: &[&EventId]) -> Result<Vec<EventId>, String>;

    /// Returns a tuple of requested state events from `event_id` and the auth chain events that
    /// relate to the.
    fn get_remote_state_for_room(
        &self,
        room_id: &RoomId,
        version: &RoomVersionId,
        event_id: &EventId,
    ) -> Result<(Vec<StateEvent>, Vec<StateEvent>), String>;
}
