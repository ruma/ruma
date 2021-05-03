use ruma_events_macros::StateEventContent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, StateEventContent)]
#[not_ruma_event(type = "m.macro.test")]
pub struct MacroTest {
    pub test: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, StateEventContent)]
#[ruma_event(event = "m.macro.test")]
pub struct MoreMacroTest {
    pub test: String,
}

fn main() {}
