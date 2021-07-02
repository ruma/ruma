use std::convert::TryFrom;

use matches::assert_matches;
use ruma_events::{room::name::RoomName, AnyInitialStateEvent, InitialStateEvent};
use serde_json::json;

#[test]
fn deserialize_initial_state_event() {
    assert_matches!(
        serde_json::from_value(json!({
            "type": "m.room.name",
            "content": { "name": "foo" }
        }))
        .unwrap(),
        AnyInitialStateEvent::RoomName(InitialStateEvent { content, state_key})
        if content.name == Some(RoomName::try_from("foo".to_owned()).unwrap())
            && state_key.is_empty()
    );
}
