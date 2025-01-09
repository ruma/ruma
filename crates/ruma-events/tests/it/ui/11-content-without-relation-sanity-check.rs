#![allow(unexpected_cfgs)]

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "m.macro.test", kind = MessageLike, without_relation)]
pub struct MacroTestContent {
    pub url: String,
    pub relates_to: Option<String>,
}

fn main() {}
