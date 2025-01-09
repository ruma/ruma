#![deny(private_interfaces, private_bounds, unnameable_types)]
#![allow(dead_code, unexpected_cfgs)]

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "m.macro.test", kind = State, state_key_type = String)]
struct MacroTestContent {
    url: String,
}

fn main() {}
