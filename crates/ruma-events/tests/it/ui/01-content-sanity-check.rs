#![allow(unexpected_cfgs)]

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "m.macro.test", kind = State, state_key_type = String)]
pub struct MacroTestContent {
    pub url: Option<String>,
}

fn main() {
    use ruma_events::{BooleanType, StateEventContent, StaticEventContent};

    assert_eq!(MacroTestContent::TYPE, "m.macro.test");
    assert!(!<MacroTestContent as StaticEventContent>::IsPrefix::as_bool());
    assert_eq!(
        MacroTestContent { url: Some("foo".to_owned()) }.event_type().to_string(),
        "m.macro.test"
    );
}
