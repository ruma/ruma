use ruma_events::{RawEventContent, StateEventContent};
use ruma_events_macros::Event;

/// State event.
#[derive(Clone, Debug, Event)]
pub struct StateEvent<C: StateEventContent> 
where
    C::Raw: RawEventContent
{
    pub not_content: C,
}

fn main() {}
