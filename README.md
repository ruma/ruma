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

// The tricky part to making this a good abstraction
trait StateStore {
    fn get_event(&self, event_id: &EventId) -> Pdu/AnyStateEvent;

    fn get_events(&self, event_ids: &[EventId]) -> Pdu/AnyStateEvent;

    fn auth_event_ids(&self, room_id: &RoomId, event_id: &EventId) -> Vec<EventId>;

    fn get_remote_state_for_room(
        &self,
        room_id: &RoomId,
        version: &RoomVersionId,
        event_id: &EventId,
    ) -> (Vec<StateEvent>, Vec<StateEvent>);

}

```
Now to be totally fair I have no real understanding of state reso
