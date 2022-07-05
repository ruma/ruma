use assert_matches::assert_matches;
use ruma_common::events::AnyInitialStateEvent;
use serde_json::json;

#[test]
fn deserialize_initial_state_event() {
    let ev = serde_json::from_value(json!({
        "type": "m.room.name",
        "content": { "name": "foo" }
    }))
    .unwrap();
    let ev = assert_matches!(ev, AnyInitialStateEvent::RoomName(ev) => ev);
    assert_eq!(ev.content.name.as_deref(), Some("foo"));
}
