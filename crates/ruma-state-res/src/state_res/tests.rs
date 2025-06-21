use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use js_int::{int, uint};
use maplit::{hashmap, hashset};
use rand::seq::SliceRandom;
use ruma_common::{
    room_version_rules::AuthorizationRules, MilliSecondsSinceUnixEpoch, OwnedEventId,
};
use ruma_events::{
    room::join_rules::{JoinRule, RoomJoinRulesEventContent},
    StateEventType, TimelineEventType,
};
use serde_json::{json, value::to_raw_value as to_raw_json_value};
use tracing::debug;

use super::{is_power_event, EventTypeExt, StateMap};
use crate::{
    test_utils::{
        alice, bob, charlie, do_check, ella, event_id, member_content_ban, member_content_join,
        room_id, to_init_pdu_event, to_pdu_event, zara, PduEvent, TestStore, INITIAL_EVENTS,
    },
    Event,
};

fn test_event_sort() {
    let _ = tracing::subscriber::set_default(tracing_subscriber::fmt().with_test_writer().finish());
    let events = INITIAL_EVENTS();

    let event_map = events
        .values()
        .map(|ev| (ev.event_type().with_state_key(ev.state_key().unwrap()), ev.clone()))
        .collect::<StateMap<_>>();

    let auth_chain: HashSet<OwnedEventId> = HashSet::new();

    let power_events = event_map
        .values()
        .filter(|&pdu| is_power_event(&**pdu))
        .map(|pdu| pdu.event_id.clone())
        .collect::<Vec<_>>();

    let sorted_power_events =
        super::sort_power_events(power_events, &auth_chain, &AuthorizationRules::V6, |id| {
            events.get(id).cloned()
        })
        .unwrap();

    let resolved_power = super::iterative_auth_checks(
        &AuthorizationRules::V6,
        &sorted_power_events,
        HashMap::new(), // unconflicted events
        |id| events.get(id).cloned(),
    )
    .expect("iterative auth check failed on resolved events");

    // don't remove any events so we know it sorts them all correctly
    let mut events_to_sort = events.keys().cloned().collect::<Vec<_>>();

    events_to_sort.shuffle(&mut rand::thread_rng());

    let power_level =
        resolved_power.get(&(StateEventType::RoomPowerLevels, "".to_owned())).cloned();

    let sorted_event_ids =
        super::mainline_sort(&events_to_sort, power_level, |id| events.get(id).cloned()).unwrap();

    assert_eq!(
        vec![
            "$CREATE:foo",
            "$IMA:foo",
            "$IPOWER:foo",
            "$IJR:foo",
            "$IMB:foo",
            "$IMC:foo",
            "$START:foo",
            "$END:foo"
        ],
        sorted_event_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>()
    );
}

#[test]
fn test_sort() {
    for _ in 0..20 {
        // since we shuffle the eventIds before we sort them introducing randomness
        // seems like we should test this a few times
        test_event_sort();
    }
}

#[test]
fn ban_vs_power_level() {
    let _ = tracing::subscriber::set_default(tracing_subscriber::fmt().with_test_writer().finish());

    let events = &[
        to_init_pdu_event(
            "PA",
            alice(),
            TimelineEventType::RoomPowerLevels,
            Some(""),
            to_raw_json_value(&json!({ "users": { alice(): 100, bob(): 50 } })).unwrap(),
        ),
        to_init_pdu_event(
            "MA",
            alice(),
            TimelineEventType::RoomMember,
            Some(alice().to_string().as_str()),
            member_content_join(),
        ),
        to_init_pdu_event(
            "MB",
            alice(),
            TimelineEventType::RoomMember,
            Some(bob().to_string().as_str()),
            member_content_ban(),
        ),
        to_init_pdu_event(
            "PB",
            bob(),
            TimelineEventType::RoomPowerLevels,
            Some(""),
            to_raw_json_value(&json!({ "users": { alice(): 100, bob(): 50 } })).unwrap(),
        ),
    ];

    let edges = vec![vec!["END", "MB", "MA", "PA", "START"], vec!["END", "PA", "PB"]]
        .into_iter()
        .map(|list| list.into_iter().map(event_id).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let expected_state_ids = vec!["PA", "MA", "MB"].into_iter().map(event_id).collect::<Vec<_>>();

    do_check(events, edges, expected_state_ids);
}

#[test]
fn topic_basic() {
    let _ = tracing::subscriber::set_default(tracing_subscriber::fmt().with_test_writer().finish());

    let events = &[
        to_init_pdu_event(
            "T1",
            alice(),
            TimelineEventType::RoomTopic,
            Some(""),
            to_raw_json_value(&json!({})).unwrap(),
        ),
        to_init_pdu_event(
            "PA1",
            alice(),
            TimelineEventType::RoomPowerLevels,
            Some(""),
            to_raw_json_value(&json!({ "users": { alice(): 100, bob(): 50 } })).unwrap(),
        ),
        to_init_pdu_event(
            "T2",
            alice(),
            TimelineEventType::RoomTopic,
            Some(""),
            to_raw_json_value(&json!({})).unwrap(),
        ),
        to_init_pdu_event(
            "PA2",
            alice(),
            TimelineEventType::RoomPowerLevels,
            Some(""),
            to_raw_json_value(&json!({ "users": { alice(): 100, bob(): 0 } })).unwrap(),
        ),
        to_init_pdu_event(
            "PB",
            bob(),
            TimelineEventType::RoomPowerLevels,
            Some(""),
            to_raw_json_value(&json!({ "users": { alice(): 100, bob(): 50 } })).unwrap(),
        ),
        to_init_pdu_event(
            "T3",
            bob(),
            TimelineEventType::RoomTopic,
            Some(""),
            to_raw_json_value(&json!({})).unwrap(),
        ),
    ];

    let edges =
        vec![vec!["END", "PA2", "T2", "PA1", "T1", "START"], vec!["END", "T3", "PB", "PA1"]]
            .into_iter()
            .map(|list| list.into_iter().map(event_id).collect::<Vec<_>>())
            .collect::<Vec<_>>();

    let expected_state_ids = vec!["PA2", "T2"].into_iter().map(event_id).collect::<Vec<_>>();

    do_check(events, edges, expected_state_ids);
}

#[test]
fn topic_reset() {
    let _ = tracing::subscriber::set_default(tracing_subscriber::fmt().with_test_writer().finish());

    let events = &[
        to_init_pdu_event(
            "T1",
            alice(),
            TimelineEventType::RoomTopic,
            Some(""),
            to_raw_json_value(&json!({})).unwrap(),
        ),
        to_init_pdu_event(
            "PA",
            alice(),
            TimelineEventType::RoomPowerLevels,
            Some(""),
            to_raw_json_value(&json!({ "users": { alice(): 100, bob(): 50 } })).unwrap(),
        ),
        to_init_pdu_event(
            "T2",
            bob(),
            TimelineEventType::RoomTopic,
            Some(""),
            to_raw_json_value(&json!({})).unwrap(),
        ),
        to_init_pdu_event(
            "MB",
            alice(),
            TimelineEventType::RoomMember,
            Some(bob().to_string().as_str()),
            member_content_ban(),
        ),
    ];

    let edges = vec![vec!["END", "MB", "T2", "PA", "T1", "START"], vec!["END", "T1"]]
        .into_iter()
        .map(|list| list.into_iter().map(event_id).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let expected_state_ids = vec!["T1", "MB", "PA"].into_iter().map(event_id).collect::<Vec<_>>();

    do_check(events, edges, expected_state_ids);
}

#[test]
fn join_rule_evasion() {
    let _ = tracing::subscriber::set_default(tracing_subscriber::fmt().with_test_writer().finish());

    let events = &[
        to_init_pdu_event(
            "JR",
            alice(),
            TimelineEventType::RoomJoinRules,
            Some(""),
            to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Private)).unwrap(),
        ),
        to_init_pdu_event(
            "ME",
            ella(),
            TimelineEventType::RoomMember,
            Some(ella().to_string().as_str()),
            member_content_join(),
        ),
    ];

    let edges = vec![vec!["END", "JR", "START"], vec!["END", "ME", "START"]]
        .into_iter()
        .map(|list| list.into_iter().map(event_id).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let expected_state_ids = vec![event_id("JR")];

    do_check(events, edges, expected_state_ids);
}

#[test]
fn offtopic_power_level() {
    let _ = tracing::subscriber::set_default(tracing_subscriber::fmt().with_test_writer().finish());

    let events = &[
        to_init_pdu_event(
            "PA",
            alice(),
            TimelineEventType::RoomPowerLevels,
            Some(""),
            to_raw_json_value(&json!({ "users": { alice(): 100, bob(): 50 } })).unwrap(),
        ),
        to_init_pdu_event(
            "PB",
            bob(),
            TimelineEventType::RoomPowerLevels,
            Some(""),
            to_raw_json_value(&json!({ "users": { alice(): 100, bob(): 50, charlie(): 50 } }))
                .unwrap(),
        ),
        to_init_pdu_event(
            "PC",
            charlie(),
            TimelineEventType::RoomPowerLevels,
            Some(""),
            to_raw_json_value(&json!({ "users": { alice(): 100, bob(): 50, charlie(): 0 } }))
                .unwrap(),
        ),
    ];

    let edges = vec![vec!["END", "PC", "PB", "PA", "START"], vec!["END", "PA"]]
        .into_iter()
        .map(|list| list.into_iter().map(event_id).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let expected_state_ids = vec!["PC"].into_iter().map(event_id).collect::<Vec<_>>();

    do_check(events, edges, expected_state_ids);
}

#[test]
fn topic_setting() {
    let _ = tracing::subscriber::set_default(tracing_subscriber::fmt().with_test_writer().finish());

    let events = &[
        to_init_pdu_event(
            "T1",
            alice(),
            TimelineEventType::RoomTopic,
            Some(""),
            to_raw_json_value(&json!({})).unwrap(),
        ),
        to_init_pdu_event(
            "PA1",
            alice(),
            TimelineEventType::RoomPowerLevels,
            Some(""),
            to_raw_json_value(&json!({ "users": { alice(): 100, bob(): 50 } })).unwrap(),
        ),
        to_init_pdu_event(
            "T2",
            alice(),
            TimelineEventType::RoomTopic,
            Some(""),
            to_raw_json_value(&json!({})).unwrap(),
        ),
        to_init_pdu_event(
            "PA2",
            alice(),
            TimelineEventType::RoomPowerLevels,
            Some(""),
            to_raw_json_value(&json!({ "users": { alice(): 100, bob(): 0 } })).unwrap(),
        ),
        to_init_pdu_event(
            "PB",
            bob(),
            TimelineEventType::RoomPowerLevels,
            Some(""),
            to_raw_json_value(&json!({ "users": { alice(): 100, bob(): 50 } })).unwrap(),
        ),
        to_init_pdu_event(
            "T3",
            bob(),
            TimelineEventType::RoomTopic,
            Some(""),
            to_raw_json_value(&json!({})).unwrap(),
        ),
        to_init_pdu_event(
            "MZ1",
            zara(),
            TimelineEventType::RoomTopic,
            Some(""),
            to_raw_json_value(&json!({})).unwrap(),
        ),
        to_init_pdu_event(
            "T4",
            alice(),
            TimelineEventType::RoomTopic,
            Some(""),
            to_raw_json_value(&json!({})).unwrap(),
        ),
    ];

    let edges = vec![
        vec!["END", "T4", "MZ1", "PA2", "T2", "PA1", "T1", "START"],
        vec!["END", "MZ1", "T3", "PB", "PA1"],
    ]
    .into_iter()
    .map(|list| list.into_iter().map(event_id).collect::<Vec<_>>())
    .collect::<Vec<_>>();

    let expected_state_ids = vec!["T4", "PA2"].into_iter().map(event_id).collect::<Vec<_>>();

    do_check(events, edges, expected_state_ids);
}

#[test]
fn test_event_map_none() {
    let _ = tracing::subscriber::set_default(tracing_subscriber::fmt().with_test_writer().finish());

    let mut store = TestStore::<PduEvent>(hashmap! {});

    // build up the DAG
    let (state_at_bob, state_at_charlie, expected) = store.set_up();

    let ev_map = store.0.clone();
    let state_sets = [state_at_bob, state_at_charlie];
    let resolved = match crate::resolve(
        &AuthorizationRules::V1,
        &state_sets,
        state_sets
            .iter()
            .map(|map| store.auth_event_ids(room_id(), map.values().cloned().collect()).unwrap())
            .collect(),
        |id| ev_map.get(id).cloned(),
    ) {
        Ok(state) => state,
        Err(e) => panic!("{e}"),
    };

    assert_eq!(expected, resolved);
}

#[test]
fn test_reverse_topological_power_sort() {
    let _ = tracing::subscriber::set_default(tracing_subscriber::fmt().with_test_writer().finish());

    let graph = hashmap! {
        event_id("l") => hashset![event_id("o")],
        event_id("m") => hashset![event_id("n"), event_id("o")],
        event_id("n") => hashset![event_id("o")],
        event_id("o") => hashset![], // "o" has zero outgoing edges but 4 incoming edges
        event_id("p") => hashset![event_id("o")],
    };

    let res = crate::reverse_topological_power_sort(&graph, |_id| {
        Ok((int!(0), MilliSecondsSinceUnixEpoch(uint!(0))))
    })
    .unwrap();

    assert_eq!(
        vec!["o", "l", "n", "m", "p"],
        res.iter()
            .map(ToString::to_string)
            .map(|s| s.replace('$', "").replace(":foo", ""))
            .collect::<Vec<_>>()
    );
}

#[test]
fn ban_with_auth_chains() {
    let _ = tracing::subscriber::set_default(tracing_subscriber::fmt().with_test_writer().finish());
    let ban = BAN_STATE_SET();

    let edges = vec![vec!["END", "MB", "PA", "START"], vec!["END", "IME", "MB"]]
        .into_iter()
        .map(|list| list.into_iter().map(event_id).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let expected_state_ids = vec!["PA", "MB"].into_iter().map(event_id).collect::<Vec<_>>();

    do_check(&ban.values().cloned().collect::<Vec<_>>(), edges, expected_state_ids);
}

#[test]
fn ban_with_auth_chains2() {
    let _ = tracing::subscriber::set_default(tracing_subscriber::fmt().with_test_writer().finish());
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
    .map(|ev| (ev.event_type().with_state_key(ev.state_key().unwrap()), ev.event_id.clone()))
    .collect::<StateMap<_>>();

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
    .map(|ev| (ev.event_type().with_state_key(ev.state_key().unwrap()), ev.event_id.clone()))
    .collect::<StateMap<_>>();

    let ev_map = &store.0;
    let state_sets = [state_set_a, state_set_b];
    let resolved = match crate::resolve(
        &AuthorizationRules::V6,
        &state_sets,
        state_sets
            .iter()
            .map(|map| store.auth_event_ids(room_id(), map.values().cloned().collect()).unwrap())
            .collect(),
        |id| ev_map.get(id).cloned(),
    ) {
        Ok(state) => state,
        Err(e) => panic!("{e}"),
    };

    debug!(
        resolved = ?resolved
            .iter()
            .map(|((ty, key), id)| format!("(({ty}{key:?}), {id})"))
            .collect::<Vec<_>>(),
        "resolved state",
    );

    let expected =
        ["$CREATE:foo", "$IJR:foo", "$PA:foo", "$IMA:foo", "$IMB:foo", "$IMC:foo", "$MB:foo"];

    for id in expected.iter().map(|i| event_id(i)) {
        // make sure our resolved events are equal to the expected list
        assert!(resolved.values().any(|eid| eid == &id) || init.contains_key(&id), "{id}");
    }
    assert_eq!(expected.len(), resolved.len());
}

#[test]
fn join_rule_with_auth_chain() {
    let join_rule = JOIN_RULE();

    let edges = vec![vec!["END", "JR", "START"], vec!["END", "IMZ", "START"]]
        .into_iter()
        .map(|list| list.into_iter().map(event_id).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let expected_state_ids = vec!["JR"].into_iter().map(event_id).collect::<Vec<_>>();

    do_check(&join_rule.values().cloned().collect::<Vec<_>>(), edges, expected_state_ids);
}

#[allow(non_snake_case)]
fn BAN_STATE_SET() -> HashMap<OwnedEventId, Arc<PduEvent>> {
    vec![
        to_pdu_event(
            "PA",
            alice(),
            TimelineEventType::RoomPowerLevels,
            Some(""),
            to_raw_json_value(&json!({ "users": { alice(): 100, bob(): 50 } })).unwrap(),
            &["CREATE", "IMA", "IPOWER"], // auth_events
            &["START"],                   // prev_events
        ),
        to_pdu_event(
            "PB",
            alice(),
            TimelineEventType::RoomPowerLevels,
            Some(""),
            to_raw_json_value(&json!({ "users": { alice(): 100, bob(): 50 } })).unwrap(),
            &["CREATE", "IMA", "IPOWER"],
            &["END"],
        ),
        to_pdu_event(
            "MB",
            alice(),
            TimelineEventType::RoomMember,
            Some(ella().as_str()),
            member_content_ban(),
            &["CREATE", "IMA", "PB"],
            &["PA"],
        ),
        to_pdu_event(
            "IME",
            ella(),
            TimelineEventType::RoomMember,
            Some(ella().as_str()),
            member_content_join(),
            &["CREATE", "IJR", "PA"],
            &["MB"],
        ),
    ]
    .into_iter()
    .map(|ev| (ev.event_id.clone(), ev))
    .collect()
}

#[allow(non_snake_case)]
fn JOIN_RULE() -> HashMap<OwnedEventId, Arc<PduEvent>> {
    vec![
        to_pdu_event(
            "JR",
            alice(),
            TimelineEventType::RoomJoinRules,
            Some(""),
            to_raw_json_value(&json!({ "join_rule": "invite" })).unwrap(),
            &["CREATE", "IMA", "IPOWER"],
            &["START"],
        ),
        to_pdu_event(
            "IMZ",
            zara(),
            TimelineEventType::RoomPowerLevels,
            Some(zara().as_str()),
            member_content_join(),
            &["CREATE", "JR", "IPOWER"],
            &["START"],
        ),
    ]
    .into_iter()
    .map(|ev| (ev.event_id.clone(), ev))
    .collect()
}

macro_rules! state_set {
    ($($kind:expr => $key:expr => $id:expr),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut x = StateMap::new();
        $(
            x.insert(($kind, $key.to_owned()), $id);
        )*
        x
    }};
}

#[test]
fn split_conflicted_state_set_conflicted_unique_state_keys() {
    let (unconflicted, conflicted) = super::split_conflicted_state_set(
        [
            state_set![StateEventType::RoomMember => "@a:hs1" => 0],
            state_set![StateEventType::RoomMember => "@b:hs1" => 1],
            state_set![StateEventType::RoomMember => "@c:hs1" => 2],
        ]
        .iter(),
    );

    assert_eq!(unconflicted, StateMap::new());
    assert_eq!(
        conflicted,
        state_set![
            StateEventType::RoomMember => "@a:hs1" => vec![0],
            StateEventType::RoomMember => "@b:hs1" => vec![1],
            StateEventType::RoomMember => "@c:hs1" => vec![2],
        ],
    );
}

#[test]
fn split_conflicted_state_set_conflicted_same_state_key() {
    let (unconflicted, mut conflicted) = super::split_conflicted_state_set(
        [
            state_set![StateEventType::RoomMember => "@a:hs1" => 0],
            state_set![StateEventType::RoomMember => "@a:hs1" => 1],
            state_set![StateEventType::RoomMember => "@a:hs1" => 2],
        ]
        .iter(),
    );

    // HashMap iteration order is random, so sort this before asserting on it
    for v in conflicted.values_mut() {
        v.sort_unstable();
    }

    assert_eq!(unconflicted, StateMap::new());
    assert_eq!(
        conflicted,
        state_set![
            StateEventType::RoomMember => "@a:hs1" => vec![0, 1, 2],
        ],
    );
}

#[test]
fn split_conflicted_state_set_unconflicted() {
    let (unconflicted, conflicted) = super::split_conflicted_state_set(
        [
            state_set![StateEventType::RoomMember => "@a:hs1" => 0],
            state_set![StateEventType::RoomMember => "@a:hs1" => 0],
            state_set![StateEventType::RoomMember => "@a:hs1" => 0],
        ]
        .iter(),
    );

    assert_eq!(
        unconflicted,
        state_set![
            StateEventType::RoomMember => "@a:hs1" => 0,
        ],
    );
    assert_eq!(conflicted, StateMap::new());
}

#[test]
fn split_conflicted_state_set_mixed() {
    let (unconflicted, conflicted) = super::split_conflicted_state_set(
        [
            state_set![StateEventType::RoomMember => "@a:hs1" => 0],
            state_set![
                StateEventType::RoomMember => "@a:hs1" => 0,
                StateEventType::RoomMember => "@b:hs1" => 1,
            ],
            state_set![
                StateEventType::RoomMember => "@a:hs1" => 0,
                StateEventType::RoomMember => "@c:hs1" => 2,
            ],
        ]
        .iter(),
    );

    assert_eq!(
        unconflicted,
        state_set![
            StateEventType::RoomMember => "@a:hs1" => 0,
        ],
    );
    assert_eq!(
        conflicted,
        state_set![
            StateEventType::RoomMember => "@b:hs1" => vec![1],
            StateEventType::RoomMember => "@c:hs1" => vec![2],
        ],
    );
}
