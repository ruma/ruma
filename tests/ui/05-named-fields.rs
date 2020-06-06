use ruma_events::StateEventContent;
use ruma_events_macros::Event;

/// State event.
#[derive(Clone, Debug, Event)]
pub struct StateEvent<C: StateEventContent>(C);

fn main() {}
