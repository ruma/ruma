use assert_matches::assert_matches;
use ruma_common::{
    events::{AnyInitialStateEvent, InitialStateEvent},
    RoomName,
};
use serde_json::json;

#[test]
fn deserialize_initial_state_event() {
    assert_matches!(
        serde_json::from_value(json!({
            "type": "m.room.name",
            "content": { "name": "foo" }
        }))
        .unwrap(),
        AnyInitialStateEvent::RoomName(InitialStateEvent { content, .. })
        if content.name == Some(Box::<RoomName>::try_from("foo").unwrap())
    );
}
