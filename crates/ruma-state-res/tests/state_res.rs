use std::sync::Arc;

use js_int::uint;
use maplit::{hashmap, hashset};
use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_events::{room::join_rules::JoinRule, EventType};
use ruma_identifiers::RoomVersionId;
use ruma_state_res::{
    self as state_res,
    test_utils::{
        alice, bob, charlie, do_check, ella, event_id, member_content_ban, member_content_join,
        room_id, to_init_pdu_event, zara, StateEvent, TestStore,
    },
    EventMap,
};
use serde_json::json;

#[test]
fn ban_vs_power_level() {
    let _ = tracing_subscriber::fmt::try_init();

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
    let _ = tracing_subscriber::fmt::try_init();

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
    let _ = tracing_subscriber::fmt::try_init();

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
    let _ = tracing_subscriber::fmt::try_init();

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
    let _ = tracing_subscriber::fmt::try_init();

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
    let _ = tracing_subscriber::fmt::try_init();

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
    let _ = tracing_subscriber::fmt::try_init();

    let mut store = TestStore::<StateEvent>(hashmap! {});

    // build up the DAG
    let (state_at_bob, state_at_charlie, expected) = store.set_up();

    let ev_map: EventMap<Arc<StateEvent>> = store.0.clone();
    let state_sets = vec![state_at_bob, state_at_charlie];
    let resolved = match state_res::resolve::<StateEvent, _>(
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
        |id| ev_map.get(id).map(Arc::clone),
    ) {
        Ok(state) => state,
        Err(e) => panic!("{}", e),
    };

    assert_eq!(expected, resolved)
}

#[test]
fn test_lexicographical_sort() {
    let _ = tracing_subscriber::fmt::try_init();

    let graph = hashmap! {
        event_id("l") => hashset![event_id("o")],
        event_id("m") => hashset![event_id("n"), event_id("o")],
        event_id("n") => hashset![event_id("o")],
        event_id("o") => hashset![], // "o" has zero outgoing edges but 4 incoming edges
        event_id("p") => hashset![event_id("o")],
    };

    let res = state_res::lexicographical_topological_sort(&graph, |id| {
        Ok((0, MilliSecondsSinceUnixEpoch(uint!(0)), id.clone()))
    })
    .unwrap();

    assert_eq!(
        vec!["o", "l", "n", "m", "p"],
        res.iter()
            .map(ToString::to_string)
            .map(|s| s.replace("$", "").replace(":foo", ""))
            .collect::<Vec<_>>()
    )
}
