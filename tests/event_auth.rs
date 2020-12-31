use std::sync::Arc;

#[rustfmt::skip] // this deletes the comments for some reason yay!
use state_res::{
    event_auth::{
        // auth_check, auth_types_for_event, can_federate, check_power_levels, check_redaction,
        valid_membership_change,
    },
    StateMap
};

mod utils;
use utils::{alice, charlie, event_id, member_content_ban, room_id, to_pdu_event, INITIAL_EVENTS};

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

    let requester = to_pdu_event(
        "HELLO",
        alice(),
        ruma::events::EventType::RoomMember,
        Some(charlie().as_str()),
        member_content_ban(),
        &vec![],
        &vec![event_id("IMC")],
    );

    assert!(valid_membership_change(
        requester.state_key().as_deref(),
        requester.sender(),
        requester.content(),
        prev,
        None,
        &auth_events
    )
    .unwrap())
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

    let requester = to_pdu_event(
        "HELLO",
        charlie(),
        ruma::events::EventType::RoomMember,
        Some(alice().as_str()),
        member_content_ban(),
        &vec![],
        &vec![event_id("IMC")],
    );

    assert!(!valid_membership_change(
        requester.state_key().as_deref(),
        requester.sender(),
        requester.content(),
        prev,
        None,
        &auth_events
    )
    .unwrap())
}
