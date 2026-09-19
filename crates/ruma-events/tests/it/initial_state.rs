use ruma_events::AnyInitialStateEvent;
use serde_json::json;
use strass::assert_let;

#[test]
fn deserialize_initial_state_event() {
    let ev = serde_json::from_value(json!({
        "type": "m.room.name",
        "content": { "name": "foo" }
    }))
    .unwrap();
    assert_let!(AnyInitialStateEvent::RoomName(ev) = ev);
    assert_eq!(ev.content.name, "foo");
}
