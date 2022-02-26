use ruma_events::StateEventContent;
use ruma_macros::Event;

/// State event.
#[derive(Clone, Debug, Event)]
pub struct StateEvent<C: StateEventContent> {
    pub not_content: C,
}

fn main() {}
