use ruma_common::events::StateEventContent;
use ruma_macros::Event;

/// State event.
#[derive(Clone, Debug, Event)]
pub struct StateEvent<C: StateEventContent>(C);

fn main() {}
