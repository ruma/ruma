### Matrix state resolution in rust!

```rust
/// StateMap is just a wrapper/deserialize target for a PDU.
struct StateEvent {
    content: serde_json::Value,
    room_id: RoomId,
    event_id: EventId,
    // ... and so on
}

/// A mapping of event type and state_key to some value `T`, usually an `EventId`.
pub type StateMap<T> = BTreeMap<(EventType, String), T>;


struct StateResolution {
    // Should any information be kept or should all of it be fetched from the
    // StateStore trait?
    event_map: BTreeMap<EventId, StateEvent>,

    // fields for temp storage during resolution??
    /// The events that conflict and their auth chains.
    conflicting_events: StateMap<Vec<EventId>>,
}

impl StateResolution {
    /// The point of this all. Resolve the conflicting set of .
    fn resolve(&mut self, events: Vec<StateMap<EventId>>) -> StateMap<EventId> { }

}

// The tricky part, making a good abstraction...
trait StateStore {
    /// Return a single event based on the EventId.
    fn get_event(&self, event_id: &EventId) -> Result<StateEvent, String>;

    /// Returns the events that correspond to the `event_ids` sorted in the same order.
    fn get_events(&self, event_ids: &[EventId]) -> Result<Vec<StateEvent>, String>;

    /// Returns a Vec of the related auth events to the given `event`.
    fn auth_event_ids(&self, room_id: &RoomId, event_ids: &[EventId]) -> Result<Vec<EventId>, String>;
}

```



The `StateStore` trait is an abstraction around what ever database your server (or maybe even client) uses to store __P__[]()ersistant __D__[]()ata __U__[]()nits.

We use `ruma`s types when deserializing any PDU or it's contents which helps avoid a lot of type checking logic [synapse](https://github.com/matrix-org/synapse) must do while authenticating event chains.