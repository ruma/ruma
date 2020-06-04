use ruma_events_macros::{FromRaw, StateEventContent};
use serde::Serialize;

#[derive(Clone, Debug, Serialize, FromRaw, StateEventContent)]
#[ruma_event(type = "m.macro.test")]
pub struct MacroTest {
    pub url: String,
}

fn main() {}
