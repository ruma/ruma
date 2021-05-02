use ruma_events_macros::StateEventContent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, StateEventContent)]
#[ruma_event(type = "m.macro.test")]
pub struct MacroTest {
    pub url: String,
}

fn main() {}
