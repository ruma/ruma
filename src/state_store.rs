use std::{collections::BTreeMap, time::SystemTime};

use petgraph::Graph;
use ruma::{
    events::{
        room::{self},
        AnyStateEvent, AnyStrippedStateEvent, AnySyncStateEvent, EventType,
    },
    identifiers::{EventId, RoomId, RoomVersionId},
};

use crate::StateEvent;

pub trait StateStore {
    /// Returns the events that correspond to the `event_ids` sorted in the same order.
    fn get_events(&self, event_ids: &[EventId]) -> Result<Vec<StateEvent>, serde_json::Error>;

    /// Returns a tuple of requested state events from `event_id` and the auth chain events that
    /// relate to the.
    fn get_remote_state_for_room(
        &self,
        room_id: &RoomId,
        version: &RoomVersionId,
        event_id: &EventId,
    ) -> Result<(Vec<StateEvent>, Vec<StateEvent>), serde_json::Error>;
}
