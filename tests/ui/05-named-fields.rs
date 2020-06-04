use ruma_events::{RawEventContent, StateEventContent};
use ruma_events_macros::Event;

/// State event.
#[derive(Clone, Debug, Event)]
pub struct StateEvent<C: StateEventContent>(C)
where
    C::Raw: RawEventContent;

fn main() {}
