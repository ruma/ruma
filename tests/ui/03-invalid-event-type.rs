use ruma_events_macros::{FromRaw, StateEventContent};
use serde::Serialize;

#[derive(Clone, Debug, Serialize, FromRaw, StateEventContent)]
#[not_ruma_event(type = "m.macro.test")]
pub struct MacroTest {
    pub test: String,
}

#[derive(Clone, Debug, Serialize, StateEventContent)]
#[ruma_event(event = "m.macro.test")]
pub struct MoreMacroTest {
    pub test: String,
}

fn main() {}
