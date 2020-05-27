use ruma_events_macros::{FromRaw, StateEventContent};
use serde::Serialize;

#[derive(Clone, Debug, Serialize, FromRaw, StateEventContent)]
pub struct MacroTest {
    pub url: String,
}

fn main() {}
