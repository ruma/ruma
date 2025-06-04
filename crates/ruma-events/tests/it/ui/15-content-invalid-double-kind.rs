#![allow(unexpected_cfgs)]

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "m.macro.test", kind = MessageLike + GlobalAccountData)]
pub struct MacroTestEventContent {
    pub url: String,
}

fn main() {}
