#![allow(unexpected_cfgs)]

use ruma_macros::EventContent;
use serde::Serialize;

#[derive(Clone, Debug, Serialize, EventContent)]
#[ruma_event(type = "m.macro.test.*", kind = GlobalAccountData)]
pub struct MacroTestContent {
    #[ruma_event(type_fragment)]
    pub frag: String,
}

fn main() {
    use ruma_events::EventContent;

    assert_eq!(
        MacroTestContent { frag: "foo".to_owned() }.event_type().to_string(),
        "m.macro.test.foo"
    );
}
