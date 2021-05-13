use ruma_events_macros::EventContent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "m.macro.test", kind = State)]
pub struct MacroTest {
    pub url: String,
}

fn main() {}
