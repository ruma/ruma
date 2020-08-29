### Matrix state resolution in rust!

```rust
/// StateMap is just a wrapper/deserialize target for a PDU.
struct StateEvent {
    content: serde_json::Value,
    origin_server_ts: SystemTime,
    sender: UserId,
    // ... and so on
}

/// A mapping of event type and state_key to some value `T`, usually an `EventId`.
pub type StateMap<T> = BTreeMap<(EventType, Option<String>), T>;

/// A mapping of `EventId` to `T`, usually a `StateEvent`.
pub type EventMap<T> = BTreeMap<EventId, T>;

struct StateResolution {
    // For now the StateResolution struct is empty. If "caching" `event_map`
    // between `resolve` calls ends up being more efficient (probably not, as this would eat memory)
    // it may have an `event_map` field. The `event_map` is all the event's
    // `StateResolution` has to know about in order to resolve state.
}

impl StateResolution {
    /// The point of this all, resolve the possibly conflicting sets of events.
    pub fn resolve(
        room_id: &RoomId,
        room_version: &RoomVersionId,
        state_sets: &[StateMap<EventId>],
        event_map: Option<EventMap<StateEvent>>,
        store: &dyn StateStore,
    ) -> Result<StateMap<EventId>, Error>;

}

// The tricky part, making a good abstraction...
trait StateStore {
    /// Return a single event based on the EventId.
    fn get_event(&self, room_id: &RoomId, event_id: &EventId) -> Result<StateEvent, Error>;

    // There are 3 methods that have default implementations `get_events`,
    // `auth_event_ids` and `auth_chain_diff`. Each could be overridden if
    //  the user has an optimization with their database of choice.
}

```



The `StateStore` trait is an abstraction around what ever database your server (or maybe even client) uses to store __P__[]()ersistant __D__[]()ata __U__[]()nits.

We use `ruma`s types when deserializing any PDU or it's contents which helps avoid a lot of type checking logic [synapse](https://github.com/matrix-org/synapse) must do while authenticating event chains.
