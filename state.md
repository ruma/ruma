Would it be possible to abstract state res into a `ruma-state-res` crate? I've been thinking about something along the lines of
```rust
// The would need to be Serialize/Deserialize to save state 
struct StateResV2 {
    resolved_events: Vec<Event>,
    state_graph: of indexes into the events field?,
    most_recent_resolved: index or ptr into the graph?,
    // fields for temp storage during resolution
    conflicting_events: Vec<Event>,
}

impl StateResV2 {
    /// The point of this all add nonconflicting events to the graph
    /// and resolve and add conflicting events.
    fn resolve(&mut self, events: Vec<Event>) -> Vec<Event> { }

}

```
Now to be totally fair I have no real understanding of state res 
