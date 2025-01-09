#![allow(unexpected_cfgs)]

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
pub struct MacroTest {
    pub url: String,
}

fn main() {}
