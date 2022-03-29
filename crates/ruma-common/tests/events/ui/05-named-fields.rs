use ruma_common::events::{EventContent, StateEventType};
use ruma_macros::Event;

/// State event.
#[derive(Clone, Debug, Event)]
pub struct StateEvent<C: EventContent<EventType = StateEventType>>(C);

fn main() {}
