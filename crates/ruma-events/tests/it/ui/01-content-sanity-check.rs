#![allow(unexpected_cfgs)]

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "m.macro.test", kind = State, state_key_type = String)]
pub struct MacroTestContent {
    pub url: String,
}

fn main() {}
