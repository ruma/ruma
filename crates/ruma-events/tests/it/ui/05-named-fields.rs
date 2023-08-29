use ruma_events::StateEventContent;
use ruma_macros::Event;

/// State event.
#[derive(Clone, Debug, Event)]
pub struct OriginalStateEvent<C: StateEventContent>(C);

fn main() {}
