# Matrix State Resolution in Rust!

```rust
/// Abstraction of a PDU so users can have their own PDU types.
pub trait Event {
    /// The `EventId` of this event.
    fn event_id(&self) -> &EventId;
    /// The `RoomId` of this event.
    fn room_id(&self) -> &RoomId;
    /// The `UserId` of this event.
    fn sender(&self) -> &UserId;
    // and so on...
}

/// A mapping of event type and state_key to some value `T`, usually an `EventId`.
pub type StateMap<T> = BTreeMap<(StateEventType, Option<String>), T>;

/// A mapping of `EventId` to `T`, usually a `OriginalStateEvent`.
pub type EventMap<T> = BTreeMap<OwnedEventId, T>;

struct StateResolution {
    // For now the StateResolution struct is empty. If "caching" `event_map`
    // between `resolve` calls ends up being more efficient (probably not, as this would eat memory)
    // it may have an `event_map` field. The `event_map` is all the events
    // `StateResolution` has to know about to resolve state.
}

impl StateResolution {
    /// The point of this all, resolve the possibly conflicting sets of events.
    pub fn resolve<E: Event>(
        room_id: &RoomId,
        room_version: &RoomVersionId,
        state_sets: &[StateMap<OwnedEventId>],
        auth_events: Vec<Vec<OwnedEventId>>,
        event_map: &mut EventMap<Arc<E>>,
    ) -> Result<StateMap<OwnedEventId>> {;
}

```



The `StateStore` trait is an abstraction around what ever database your server (or maybe even client) uses to store **P**ersistent **D**ata **U**nits.

We use `ruma`s types when deserializing any PDU or it's contents which helps avoid a lot of type checking logic [synapse](https://github.com/matrix-org/synapse) must do while authenticating event chains.
