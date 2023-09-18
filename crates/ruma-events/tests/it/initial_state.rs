use assert_matches2::assert_matches;
use ruma_events::AnyInitialStateEvent;
use serde_json::json;

#[test]
fn deserialize_initial_state_event() {
    let ev = serde_json::from_value(json!({
        "type": "m.room.name",
        "content": { "name": "foo" }
    }))
    .unwrap();
    assert_matches!(ev, AnyInitialStateEvent::RoomName(ev));
    assert_eq!(ev.content.name, "foo");
}
