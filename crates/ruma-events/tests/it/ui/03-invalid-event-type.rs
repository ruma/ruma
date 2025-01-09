#![allow(unexpected_cfgs)]

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[not_ruma_event(type = "m.macro.test", kind = State)]
pub struct MacroTest {
    pub test: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(event = "m.macro.test", kind = State)]
pub struct MoreMacroTest {
    pub test: String,
}

fn main() {}
