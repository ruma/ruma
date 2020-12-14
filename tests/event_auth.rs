use std::sync::Arc;

#[rustfmt::skip] // this deletes the comments for some reason yay!
use state_res::{
    event_auth::{
        // auth_check, auth_types_for_event, can_federate, check_power_levels, check_redaction,
        valid_membership_change,
    },
    Requester, StateMap
};

mod utils;
use utils::{alice, charlie, event_id, member_content_ban, room_id, INITIAL_EVENTS};

#[test]
fn test_ban_pass() {
    let events = INITIAL_EVENTS();

    let prev = events
        .values()
        .find(|ev| ev.event_id().as_str().contains("IMC"))
        .map(Arc::clone);

    let auth_events = events
        .values()
        .map(|ev| ((ev.kind(), ev.state_key()), Arc::clone(ev)))
        .collect::<StateMap<_>>();

    let requester = Requester {
        prev_event_ids: vec![event_id("IMC")],
        room_id: &room_id(),
        content: &member_content_ban(),
        state_key: Some(charlie().to_string()),
        sender: &alice(),
    };

    assert!(valid_membership_change(requester, prev, None, &auth_events).unwrap())
}

#[test]
fn test_ban_fail() {
    let events = INITIAL_EVENTS();

    let prev = events
        .values()
        .find(|ev| ev.event_id().as_str().contains("IMC"))
        .map(Arc::clone);

    let auth_events = events
        .values()
        .map(|ev| ((ev.kind(), ev.state_key()), Arc::clone(ev)))
        .collect::<StateMap<_>>();

    let requester = Requester {
        prev_event_ids: vec![event_id("IMC")],
        room_id: &room_id(),
        content: &member_content_ban(),
        state_key: Some(alice().to_string()),
        sender: &charlie(),
    };

    assert!(!valid_membership_change(requester, prev, None, &auth_events).unwrap())
}
