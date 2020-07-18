Would it be possible to abstract state res into a `ruma-state-res` crate? I've been thinking about something along the lines of
```rust
// The would need to be Serialize/Deserialize to save state 
struct StateResV2 {
    // Should any information be kept or should all of it be fetched from the
    // StateStore trait?,
    state_graph: Something,

    // fields for temp storage during resolution??
    conflicting_events: StateMap<Vec<EventId>>,
}

impl StateResV2 {
    /// The point of this all add nonconflicting events to the graph
    /// and resolve and add conflicting events.
    fn resolve(&mut self, events: Vec<StateMap<EventId>>) -> StateMap<EventId> { }

}

// The tricky part of making a good abstraction
trait StateStore {
    /// Return a single event based on the EventId.
    fn get_event(&self, event_id: &EventId) -> Result<StateEvent, String>;

    /// Returns the events that correspond to the `event_ids` sorted in the same order.
    fn get_events(&self, event_ids: &[EventId]) -> Result<Vec<StateEvent>, String>;

    /// Returns a Vec of the related auth events to the given `event`.
    fn auth_event_ids(&self, room_id: &RoomId, event_id: &EventId) -> Result<Vec<EventId>, String>;

    /// Returns a tuple of requested state events from `event_id` and the auth chain events that
    /// they relate to the.
    fn get_remote_state_for_room(
        &self,
        room_id: &RoomId,
        version: &RoomVersionId,
        event_id: &EventId,
    ) -> Result<(Vec<StateEvent>, Vec<StateEvent>), String>;

}

```
