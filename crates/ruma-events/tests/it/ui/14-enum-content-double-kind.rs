#![allow(unexpected_cfgs)]

use ruma_macros::event_enum;

mod event {
    use ruma_macros::EventContent;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
    #[ruma_event(type = "m.macro.test", kind = RoomAccountData + GlobalAccountData)]
    pub struct MacroTestEventContent {
        pub url: String,
    }
}

event_enum! {
    /// Any global account data event.
    enum GlobalAccountData {
        "m.macro.test" => event,
    }

    /// Any room account data event.
    enum RoomAccountData {
        "m.macro.test" => event,
    }
}

fn main() {
    let content = event::MacroTestEventContent { url: "http://localhost".to_owned() };

    // Both traits are implemented for the content.
    assert_eq!(
        ruma_events::GlobalAccountDataEventContent::event_type(&content).to_string(),
        "m.macro.test"
    );
    assert_eq!(
        ruma_events::RoomAccountDataEventContent::event_type(&content).to_string(),
        "m.macro.test"
    );

    // Both event type aliases are created, and they work with the enum variants.
    let _ = AnyGlobalAccountDataEvent::MacroTest(event::GlobalMacroTestEvent {
        content: content.clone(),
    });
    let _ = AnyRoomAccountDataEvent::MacroTest(event::RoomMacroTestEvent { content });

    // Both event type enums variants are created.
    assert_eq!(GlobalAccountDataEventType::MacroTest.to_string(), "m.macro.test");
    assert_eq!(RoomAccountDataEventType::MacroTest.to_string(), "m.macro.test");
}

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrivOwnedStr(Box<str>);
