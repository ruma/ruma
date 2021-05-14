use std::sync::Arc;

use js_int::uint;
use maplit::{btreemap, btreeset};
use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_events::{room::join_rules::JoinRule, EventType};
use ruma_identifiers::{EventId, RoomVersionId};
use ruma_state_res::{EventMap, StateMap, StateResolution};
use serde_json::json;
use tracing_subscriber as tracer;

mod utils;
use utils::{
    alice, bob, charlie, do_check, ella, event_id, member_content_ban, member_content_join,
    room_id, to_init_pdu_event, to_pdu_event, zara, StateEvent, TestStore, LOGGER,
};

#[test]
fn ban_vs_power_level() {
    let events = &[
        to_init_pdu_event(
            "PA",
            alice(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({ "users": { alice(): 100, bob(): 50 } }),
        ),
        to_init_pdu_event(
            "MA",
            alice(),
            EventType::RoomMember,
            Some(alice().to_string().as_str()),
            member_content_join(),
        ),
        to_init_pdu_event(
            "MB",
            alice(),
            EventType::RoomMember,
            Some(bob().to_string().as_str()),
            member_content_ban(),
        ),
        to_init_pdu_event(
            "PB",
            bob(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({ "users": { alice(): 100, bob(): 50 } }),
        ),
    ];

    let edges = vec![vec!["END", "MB", "MA", "PA", "START"], vec!["END", "PA", "PB"]]
        .into_iter()
        .map(|list| list.into_iter().map(event_id).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let expected_state_ids = vec!["PA", "MA", "MB"].into_iter().map(event_id).collect::<Vec<_>>();

    do_check(events, edges, expected_state_ids)
}

#[test]
fn topic_basic() {
    let events = &[
        to_init_pdu_event("T1", alice(), EventType::RoomTopic, Some(""), json!({})),
        to_init_pdu_event(
            "PA1",
            alice(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({ "users": { alice(): 100, bob(): 50 } }),
        ),
        to_init_pdu_event("T2", alice(), EventType::RoomTopic, Some(""), json!({})),
        to_init_pdu_event(
            "PA2",
            alice(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({ "users": { alice(): 100, bob(): 0 } }),
        ),
        to_init_pdu_event(
            "PB",
            bob(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({ "users": { alice(): 100, bob(): 50 } }),
        ),
        to_init_pdu_event("T3", bob(), EventType::RoomTopic, Some(""), json!({})),
    ];

    let edges =
        vec![vec!["END", "PA2", "T2", "PA1", "T1", "START"], vec!["END", "T3", "PB", "PA1"]]
            .into_iter()
            .map(|list| list.into_iter().map(event_id).collect::<Vec<_>>())
            .collect::<Vec<_>>();

    let expected_state_ids = vec!["PA2", "T2"].into_iter().map(event_id).collect::<Vec<_>>();

    do_check(events, edges, expected_state_ids)
}

#[test]
fn topic_reset() {
    let events = &[
        to_init_pdu_event("T1", alice(), EventType::RoomTopic, Some(""), json!({})),
        to_init_pdu_event(
            "PA",
            alice(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({ "users": { alice(): 100, bob(): 50 } }),
        ),
        to_init_pdu_event("T2", bob(), EventType::RoomTopic, Some(""), json!({})),
        to_init_pdu_event(
            "MB",
            alice(),
            EventType::RoomMember,
            Some(bob().to_string().as_str()),
            member_content_ban(),
        ),
    ];

    let edges = vec![vec!["END", "MB", "T2", "PA", "T1", "START"], vec!["END", "T1"]]
        .into_iter()
        .map(|list| list.into_iter().map(event_id).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let expected_state_ids = vec!["T1", "MB", "PA"].into_iter().map(event_id).collect::<Vec<_>>();

    do_check(events, edges, expected_state_ids)
}

#[test]
fn join_rule_evasion() {
    let events = &[
        to_init_pdu_event(
            "JR",
            alice(),
            EventType::RoomJoinRules,
            Some(""),
            json!({ "join_rule": JoinRule::Private }),
        ),
        to_init_pdu_event(
            "ME",
            ella(),
            EventType::RoomMember,
            Some(ella().to_string().as_str()),
            member_content_join(),
        ),
    ];

    let edges = vec![vec!["END", "JR", "START"], vec!["END", "ME", "START"]]
        .into_iter()
        .map(|list| list.into_iter().map(event_id).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let expected_state_ids = vec![event_id("JR")];

    do_check(events, edges, expected_state_ids)
}

#[test]
fn offtopic_power_level() {
    let events = &[
        to_init_pdu_event(
            "PA",
            alice(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({ "users": { alice(): 100, bob(): 50 } }),
        ),
        to_init_pdu_event(
            "PB",
            bob(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({ "users": { alice(): 100, bob(): 50, charlie(): 50 } }),
        ),
        to_init_pdu_event(
            "PC",
            charlie(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({ "users": { alice(): 100, bob(): 50, charlie(): 0 } }),
        ),
    ];

    let edges = vec![vec!["END", "PC", "PB", "PA", "START"], vec!["END", "PA"]]
        .into_iter()
        .map(|list| list.into_iter().map(event_id).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let expected_state_ids = vec!["PC"].into_iter().map(event_id).collect::<Vec<_>>();

    do_check(events, edges, expected_state_ids)
}

#[test]
fn topic_setting() {
    let events = &[
        to_init_pdu_event("T1", alice(), EventType::RoomTopic, Some(""), json!({})),
        to_init_pdu_event(
            "PA1",
            alice(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({ "users": { alice(): 100, bob(): 50 } }),
        ),
        to_init_pdu_event("T2", alice(), EventType::RoomTopic, Some(""), json!({})),
        to_init_pdu_event(
            "PA2",
            alice(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({ "users": { alice(): 100, bob(): 0 } }),
        ),
        to_init_pdu_event(
            "PB",
            bob(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({ "users": { alice(): 100, bob(): 50 } }),
        ),
        to_init_pdu_event("T3", bob(), EventType::RoomTopic, Some(""), json!({})),
        to_init_pdu_event("MZ1", zara(), EventType::RoomTopic, Some(""), json!({})),
        to_init_pdu_event("T4", alice(), EventType::RoomTopic, Some(""), json!({})),
    ];

    let edges = vec![
        vec!["END", "T4", "MZ1", "PA2", "T2", "PA1", "T1", "START"],
        vec!["END", "MZ1", "T3", "PB", "PA1"],
    ]
    .into_iter()
    .map(|list| list.into_iter().map(event_id).collect::<Vec<_>>())
    .collect::<Vec<_>>();

    let expected_state_ids = vec!["T4", "PA2"].into_iter().map(event_id).collect::<Vec<_>>();

    do_check(events, edges, expected_state_ids)
}

#[test]
fn test_event_map_none() {
    // This is the only test that does not use `do_check` so we
    // have to start the logger if this is what we're running.
    //
    // to activate logging use `RUST_LOG=debug cargo t one_test_only`
    let _ = LOGGER
        .call_once(|| tracer::fmt().with_env_filter(tracer::EnvFilter::from_default_env()).init());

    let mut store = TestStore::<StateEvent>(btreemap! {});

    // build up the DAG
    let (state_at_bob, state_at_charlie, expected) = store.set_up();

    let mut ev_map: EventMap<Arc<StateEvent>> = store.0.clone();
    let state_sets = vec![state_at_bob, state_at_charlie];
    let resolved = match StateResolution::resolve::<StateEvent>(
        &room_id(),
        &RoomVersionId::Version2,
        &state_sets,
        state_sets
            .iter()
            .map(|map| {
                store
                    .auth_event_ids(&room_id(), &map.values().cloned().collect::<Vec<_>>())
                    .unwrap()
            })
            .collect(),
        &mut ev_map,
    ) {
        Ok(state) => state,
        Err(e) => panic!("{}", e),
    };

    assert_eq!(expected, resolved)
}

#[test]
fn test_lexicographical_sort() {
    let graph = btreemap! {
        event_id("l") => btreeset![event_id("o")],
        event_id("m") => btreeset![event_id("n"), event_id("o")],
        event_id("n") => btreeset![event_id("o")],
        event_id("o") => btreeset![], // "o" has zero outgoing edges but 4 incoming edges
        event_id("p") => btreeset![event_id("o")],
    };

    let res = StateResolution::lexicographical_topological_sort(&graph, |id| {
        (0, MilliSecondsSinceUnixEpoch(uint!(0)), id.clone())
    });

    assert_eq!(
        vec!["o", "l", "n", "m", "p"],
        res.iter()
            .map(ToString::to_string)
            .map(|s| s.replace("$", "").replace(":foo", ""))
            .collect::<Vec<_>>()
    )
}

// A StateStore implementation for testing
//
//
impl TestStore<StateEvent> {
    pub fn set_up(&mut self) -> (StateMap<EventId>, StateMap<EventId>, StateMap<EventId>) {
        let create_event = to_pdu_event::<EventId>(
            "CREATE",
            alice(),
            EventType::RoomCreate,
            Some(""),
            json!({ "creator": alice() }),
            &[],
            &[],
        );
        let cre = create_event.event_id().clone();
        self.0.insert(cre.clone(), Arc::clone(&create_event));

        let alice_mem = to_pdu_event(
            "IMA",
            alice(),
            EventType::RoomMember,
            Some(alice().to_string().as_str()),
            member_content_join(),
            &[cre.clone()],
            &[cre.clone()],
        );
        self.0.insert(alice_mem.event_id().clone(), Arc::clone(&alice_mem));

        let join_rules = to_pdu_event(
            "IJR",
            alice(),
            EventType::RoomJoinRules,
            Some(""),
            json!({ "join_rule": JoinRule::Public }),
            &[cre.clone(), alice_mem.event_id().clone()],
            &[alice_mem.event_id().clone()],
        );
        self.0.insert(join_rules.event_id().clone(), join_rules.clone());

        // Bob and Charlie join at the same time, so there is a fork
        // this will be represented in the state_sets when we resolve
        let bob_mem = to_pdu_event(
            "IMB",
            bob(),
            EventType::RoomMember,
            Some(bob().to_string().as_str()),
            member_content_join(),
            &[cre.clone(), join_rules.event_id().clone()],
            &[join_rules.event_id().clone()],
        );
        self.0.insert(bob_mem.event_id().clone(), bob_mem.clone());

        let charlie_mem = to_pdu_event(
            "IMC",
            charlie(),
            EventType::RoomMember,
            Some(charlie().to_string().as_str()),
            member_content_join(),
            &[cre, join_rules.event_id().clone()],
            &[join_rules.event_id().clone()],
        );
        self.0.insert(charlie_mem.event_id().clone(), charlie_mem.clone());

        let state_at_bob = [&create_event, &alice_mem, &join_rules, &bob_mem]
            .iter()
            .map(|e| ((e.kind(), e.state_key()), e.event_id().clone()))
            .collect::<StateMap<_>>();

        let state_at_charlie = [&create_event, &alice_mem, &join_rules, &charlie_mem]
            .iter()
            .map(|e| ((e.kind(), e.state_key()), e.event_id().clone()))
            .collect::<StateMap<_>>();

        let expected = [&create_event, &alice_mem, &join_rules, &bob_mem, &charlie_mem]
            .iter()
            .map(|e| ((e.kind(), e.state_key()), e.event_id().clone()))
            .collect::<StateMap<_>>();

        (state_at_bob, state_at_charlie, expected)
    }
}
