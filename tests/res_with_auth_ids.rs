#![allow(clippy::or_fun_call, clippy::expect_fun_call)]

use std::{collections::BTreeMap, sync::Arc};

use ruma::{
    events::EventType,
    identifiers::{EventId, RoomVersionId},
};
use serde_json::json;
use state_res::{StateEvent, StateMap, StateResolution};

mod utils;
use utils::{
    alice, bob, do_check, ella, event_id, member_content_ban, member_content_join, room_id,
    to_pdu_event, zara, TestStore, INITIAL_EVENTS,
};

#[test]
fn ban_with_auth_chains() {
    let ban = BAN_STATE_SET();

    let edges = vec![vec!["END", "MB", "PA", "START"], vec!["END", "IME", "MB"]]
        .into_iter()
        .map(|list| list.into_iter().map(event_id).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let expected_state_ids = vec!["PA", "MB", "END"]
        .into_iter()
        .map(event_id)
        .collect::<Vec<_>>();

    do_check(
        &ban.values().cloned().collect::<Vec<_>>(),
        edges,
        expected_state_ids,
    );
}

// Sanity check that the store is able to fetch auth chain and such
#[test]
fn base_with_auth_chains() {
    let store = TestStore(INITIAL_EVENTS());

    let resolved: BTreeMap<_, EventId> =
        match StateResolution::resolve(&room_id(), &RoomVersionId::Version6, &[], None, &store) {
            Ok(state) => state,
            Err(e) => panic!("{}", e),
        };

    let resolved = resolved
        .values()
        .cloned()
        .chain(INITIAL_EVENTS().values().map(|e| e.event_id()))
        .collect::<Vec<_>>();

    let expected = vec![
        "$CREATE:foo",
        "$IJR:foo",
        "$IPOWER:foo",
        "$IMA:foo",
        "$IMB:foo",
        "$IMC:foo",
        "START",
        "END",
    ];
    for id in expected.iter().map(|i| event_id(i)) {
        // make sure our resolved events are equal to the expected list
        assert!(resolved.iter().any(|eid| eid == &id), "{}", id)
    }
    assert_eq!(expected.len(), resolved.len())
}

#[test]
fn ban_with_auth_chains2() {
    let init = INITIAL_EVENTS();
    let ban = BAN_STATE_SET();

    let mut inner = init.clone();
    inner.extend(ban);
    let store = TestStore(inner.clone());

    let state_set_a = [
        inner.get(&event_id("CREATE")).unwrap(),
        inner.get(&event_id("IJR")).unwrap(),
        inner.get(&event_id("IMA")).unwrap(),
        inner.get(&event_id("IMB")).unwrap(),
        inner.get(&event_id("IMC")).unwrap(),
        inner.get(&event_id("MB")).unwrap(),
        inner.get(&event_id("PA")).unwrap(),
    ]
    .iter()
    .map(|ev| ((ev.kind(), ev.state_key()), ev.event_id()))
    .collect::<BTreeMap<_, _>>();

    let state_set_b = [
        inner.get(&event_id("CREATE")).unwrap(),
        inner.get(&event_id("IJR")).unwrap(),
        inner.get(&event_id("IMA")).unwrap(),
        inner.get(&event_id("IMB")).unwrap(),
        inner.get(&event_id("IMC")).unwrap(),
        inner.get(&event_id("IME")).unwrap(),
        inner.get(&event_id("PA")).unwrap(),
    ]
    .iter()
    .map(|ev| ((ev.kind(), ev.state_key()), ev.event_id()))
    .collect::<StateMap<_>>();

    let resolved: StateMap<EventId> = match StateResolution::resolve(
        &room_id(),
        &RoomVersionId::Version6,
        &[state_set_a, state_set_b],
        None,
        &store,
    ) {
        Ok(state) => state,
        Err(e) => panic!("{}", e),
    };

    tracing::debug!(
        "{:#?}",
        resolved
            .iter()
            .map(|((ty, key), id)| format!("(({}{:?}), {})", ty, key, id))
            .collect::<Vec<_>>()
    );

    let expected = vec![
        "$CREATE:foo",
        "$IJR:foo",
        "$PA:foo",
        "$IMA:foo",
        "$IMB:foo",
        "$IMC:foo",
        "$MB:foo",
    ];

    for id in expected.iter().map(|i| event_id(i)) {
        // make sure our resolved events are equal to the expected list
        assert!(
            resolved.values().any(|eid| eid == &id) || init.contains_key(&id),
            "{}",
            id
        )
    }
    assert_eq!(expected.len(), resolved.len())
}

#[test]
fn join_rule_with_auth_chain() {
    let join_rule = JOIN_RULE();

    let edges = vec![vec!["END", "JR", "START"], vec!["END", "IMZ", "START"]]
        .into_iter()
        .map(|list| list.into_iter().map(event_id).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let expected_state_ids = vec!["JR", "END"]
        .into_iter()
        .map(event_id)
        .collect::<Vec<_>>();

    do_check(
        &join_rule.values().cloned().collect::<Vec<_>>(),
        edges,
        expected_state_ids,
    );
}

#[allow(non_snake_case)]
fn BAN_STATE_SET() -> BTreeMap<EventId, Arc<StateEvent>> {
    vec![
        to_pdu_event(
            "PA",
            alice(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({"users": {alice(): 100, bob(): 50}}),
            &["CREATE", "IMA", "IPOWER"], // auth_events
            &["START"],                   // prev_events
        ),
        to_pdu_event(
            "PB",
            alice(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({"users": {alice(): 100, bob(): 50}}),
            &["CREATE", "IMA", "IPOWER"],
            &["END"],
        ),
        to_pdu_event(
            "MB",
            alice(),
            EventType::RoomMember,
            Some(ella().as_str()),
            member_content_ban(),
            &["CREATE", "IMA", "PB"],
            &["PA"],
        ),
        to_pdu_event(
            "IME",
            ella(),
            EventType::RoomMember,
            Some(ella().as_str()),
            member_content_join(),
            &["CREATE", "IJR", "PA"],
            &["MB"],
        ),
    ]
    .into_iter()
    .map(|ev| (ev.event_id(), ev))
    .collect()
}

#[allow(non_snake_case)]
fn JOIN_RULE() -> BTreeMap<EventId, Arc<StateEvent>> {
    vec![
        to_pdu_event(
            "JR",
            alice(),
            EventType::RoomJoinRules,
            Some(""),
            json!({"join_rule": "invite"}),
            &["CREATE", "IMA", "IPOWER"],
            &["START"],
        ),
        to_pdu_event(
            "IMZ",
            zara(),
            EventType::RoomPowerLevels,
            Some(zara().as_str()),
            member_content_join(),
            &["CREATE", "JR", "IPOWER"],
            &["START"],
        ),
    ]
    .into_iter()
    .map(|ev| (ev.event_id(), ev))
    .collect()
}
