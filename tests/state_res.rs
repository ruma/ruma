use std::{collections::BTreeMap, convert::TryFrom, sync::Arc, time::UNIX_EPOCH};

use maplit::btreemap;
use ruma::{
    events::{
        room::{
            join_rules::JoinRule,
            member::{MemberEventContent, MembershipState},
        },
        EventType,
    },
    identifiers::{EventId, RoomId, RoomVersionId, UserId},
};
use serde_json::{json, Value as JsonValue};
use state_res::{Error, Result, StateEvent, StateMap, StateResolution, StateStore};
use tracing_subscriber as tracer;

use std::sync::Once;

static LOGGER: Once = Once::new();

static mut SERVER_TIMESTAMP: i32 = 0;

fn event_id(id: &str) -> EventId {
    if id.contains('$') {
        return EventId::try_from(id).unwrap();
    }
    EventId::try_from(format!("${}:foo", id)).unwrap()
}

fn alice() -> UserId {
    UserId::try_from("@alice:foo").unwrap()
}
fn bob() -> UserId {
    UserId::try_from("@bob:foo").unwrap()
}
fn charlie() -> UserId {
    UserId::try_from("@charlie:foo").unwrap()
}
fn ella() -> UserId {
    UserId::try_from("@ella:foo").unwrap()
}
fn zera() -> UserId {
    UserId::try_from("@zera:foo").unwrap()
}

fn room_id() -> RoomId {
    RoomId::try_from("!test:foo").unwrap()
}

fn member_content_ban() -> JsonValue {
    serde_json::to_value(MemberEventContent {
        membership: MembershipState::Ban,
        displayname: None,
        avatar_url: None,
        is_direct: None,
        third_party_invite: None,
    })
    .unwrap()
}
fn member_content_join() -> JsonValue {
    serde_json::to_value(MemberEventContent {
        membership: MembershipState::Join,
        displayname: None,
        avatar_url: None,
        is_direct: None,
        third_party_invite: None,
    })
    .unwrap()
}

fn to_pdu_event<S>(
    id: &str,
    sender: UserId,
    ev_type: EventType,
    state_key: Option<&str>,
    content: JsonValue,
    auth_events: &[S],
    prev_events: &[S],
) -> Arc<StateEvent>
where
    S: AsRef<str>,
{
    let ts = unsafe {
        let ts = SERVER_TIMESTAMP;
        // increment the "origin_server_ts" value
        SERVER_TIMESTAMP += 1;
        ts
    };
    let id = if id.contains('$') {
        id.to_string()
    } else {
        format!("${}:foo", id)
    };
    let auth_events = auth_events
        .iter()
        .map(AsRef::as_ref)
        .map(event_id)
        .collect::<Vec<_>>();
    let prev_events = prev_events
        .iter()
        .map(AsRef::as_ref)
        .map(event_id)
        .collect::<Vec<_>>();

    let json = if let Some(state_key) = state_key {
        json!({
            "auth_events": auth_events,
            "prev_events": prev_events,
            "event_id": id,
            "sender": sender,
            "type": ev_type,
            "state_key": state_key,
            "content": content,
            "origin_server_ts": ts,
            "room_id": room_id(),
            "origin": "foo",
            "depth": 0,
            "hashes": { "sha256": "hello" },
            "signatures": {},
        })
    } else {
        json!({
            "auth_events": auth_events,
            "prev_events": prev_events,
            "event_id": id,
            "sender": sender,
            "type": ev_type,
            "content": content,
            "origin_server_ts": ts,
            "room_id": room_id(),
            "origin": "foo",
            "depth": 0,
            "hashes": { "sha256": "hello" },
            "signatures": {},
        })
    };
    Arc::new(serde_json::from_value(json).unwrap())
}

fn to_init_pdu_event(
    id: &str,
    sender: UserId,
    ev_type: EventType,
    state_key: Option<&str>,
    content: JsonValue,
) -> Arc<StateEvent> {
    let ts = unsafe {
        let ts = SERVER_TIMESTAMP;
        // increment the "origin_server_ts" value
        SERVER_TIMESTAMP += 1;
        ts
    };
    let id = if id.contains('$') {
        id.to_string()
    } else {
        format!("${}:foo", id)
    };

    let json = if let Some(state_key) = state_key {
        json!({
            "auth_events": [],
            "prev_events": [],
            "event_id": id,
            "sender": sender,
            "type": ev_type,
            "state_key": state_key,
            "content": content,
            "origin_server_ts": ts,
            "room_id": room_id(),
            "origin": "foo",
            "depth": 0,
            "hashes": { "sha256": "hello" },
            "signatures": {},
        })
    } else {
        json!({
            "auth_events": [],
            "prev_events": [],
            "event_id": id,
            "sender": sender,
            "type": ev_type,
            "content": content,
            "origin_server_ts": ts,
            "room_id": room_id(),
            "origin": "foo",
            "depth": 0,
            "hashes": { "sha256": "hello" },
            "signatures": {},
        })
    };
    Arc::new(serde_json::from_value(json).unwrap())
}

// all graphs start with these input events
#[allow(non_snake_case)]
fn INITIAL_EVENTS() -> BTreeMap<EventId, Arc<StateEvent>> {
    vec![
        to_init_pdu_event(
            "CREATE",
            alice(),
            EventType::RoomCreate,
            Some(""),
            json!({ "creator": alice() }),
        ),
        to_init_pdu_event(
            "IMA",
            alice(),
            EventType::RoomMember,
            Some(alice().to_string().as_str()),
            member_content_join(),
        ),
        to_init_pdu_event(
            "IPOWER",
            alice(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({"users": {alice().to_string(): 100}}),
        ),
        to_init_pdu_event(
            "IJR",
            alice(),
            EventType::RoomJoinRules,
            Some(""),
            json!({ "join_rule": JoinRule::Public }),
        ),
        to_init_pdu_event(
            "IMB",
            bob(),
            EventType::RoomMember,
            Some(bob().to_string().as_str()),
            member_content_join(),
        ),
        to_init_pdu_event(
            "IMC",
            charlie(),
            EventType::RoomMember,
            Some(charlie().to_string().as_str()),
            member_content_join(),
        ),
        to_init_pdu_event(
            "IMZ",
            zera(),
            EventType::RoomMember,
            Some(zera().to_string().as_str()),
            member_content_join(),
        ),
        to_init_pdu_event("START", zera(), EventType::RoomTopic, Some(""), json!({})),
        to_init_pdu_event("END", zera(), EventType::RoomTopic, Some(""), json!({})),
    ]
    .into_iter()
    .map(|ev| (ev.event_id(), ev))
    .collect()
}

#[allow(non_snake_case)]
fn INITIAL_EDGES() -> Vec<EventId> {
    vec![
        "START", "IMZ", "IMC", "IMB", "IJR", "IPOWER", "IMA", "CREATE",
    ]
    .into_iter()
    .map(event_id)
    .collect::<Vec<_>>()
}

fn do_check(
    events: &[Arc<StateEvent>],
    edges: Vec<Vec<EventId>>,
    expected_state_ids: Vec<EventId>,
) {
    // to activate logging use `RUST_LOG=debug cargo t one_test_only`
    let _ = LOGGER.call_once(|| {
        tracer::fmt()
            .with_env_filter(tracer::EnvFilter::from_default_env())
            .init()
    });

    let mut store = TestStore(
        INITIAL_EVENTS()
            .values()
            .chain(events)
            .map(|ev| (ev.event_id(), ev.clone()))
            .collect(),
    );

    // This will be lexi_topo_sorted for resolution
    let mut graph = BTreeMap::new();
    // this is the same as in `resolve` event_id -> StateEvent
    let mut fake_event_map = BTreeMap::new();

    // create the DB of events that led up to this point
    // TODO maybe clean up some of these clones it is just tests but...
    for ev in INITIAL_EVENTS().values().chain(events) {
        graph.insert(ev.event_id(), vec![]);
        fake_event_map.insert(ev.event_id(), ev.clone());
    }

    for pair in INITIAL_EDGES().windows(2) {
        if let [a, b] = &pair {
            graph
                .entry(a.clone())
                .or_insert_with(Vec::new)
                .push(b.clone());
        }
    }

    for edge_list in edges {
        for pair in edge_list.windows(2) {
            if let [a, b] = &pair {
                graph
                    .entry(a.clone())
                    .or_insert_with(Vec::new)
                    .push(b.clone());
            }
        }
    }

    // event_id -> StateEvent
    let mut event_map: BTreeMap<EventId, Arc<StateEvent>> = BTreeMap::new();
    // event_id -> StateMap<EventId>
    let mut state_at_event: BTreeMap<EventId, StateMap<EventId>> = BTreeMap::new();

    // resolve the current state and add it to the state_at_event map then continue
    // on in "time"
    for node in
        StateResolution::lexicographical_topological_sort(&graph, |id| (0, UNIX_EPOCH, id.clone()))
    {
        let fake_event = fake_event_map.get(&node).unwrap();
        let event_id = fake_event.event_id();

        let prev_events = graph.get(&node).unwrap();

        let state_before: StateMap<EventId> = if prev_events.is_empty() {
            BTreeMap::new()
        } else if prev_events.len() == 1 {
            state_at_event.get(&prev_events[0]).unwrap().clone()
        } else {
            let state_sets = prev_events
                .iter()
                .filter_map(|k| state_at_event.get(k))
                .cloned()
                .collect::<Vec<_>>();

            tracing::debug!(
                "{:#?}",
                state_sets
                    .iter()
                    .map(|map| map
                        .iter()
                        .map(|((ty, key), id)| format!("(({}{:?}), {})", ty, key, id))
                        .collect::<Vec<_>>())
                    .collect::<Vec<_>>()
            );

            let resolved = StateResolution::resolve(
                &room_id(),
                &RoomVersionId::Version6,
                &state_sets,
                Some(event_map.clone()),
                &store,
            );
            match resolved {
                Ok(state) => state,
                Err(e) => panic!("resolution for {} failed: {}", node, e),
            }
        };

        let mut state_after = state_before.clone();

        let ty = fake_event.kind().clone();
        let key = fake_event.state_key().clone();
        state_after.insert((ty, key), event_id.clone());

        let auth_types = state_res::auth_types_for_event(
            fake_event.kind(),
            fake_event.sender(),
            Some(fake_event.state_key()),
            fake_event.content().clone(),
        );

        let mut auth_events = vec![];
        for key in auth_types {
            if state_before.contains_key(&key) {
                auth_events.push(state_before[&key].clone())
            }
        }

        // TODO The event is just remade, adding the auth_events and prev_events here
        // UPDATE: the `to_pdu_event` was split into `init` and the fn below, could be better
        let e = fake_event;
        let ev_id = e.event_id();
        let event = to_pdu_event(
            &e.event_id().to_string(),
            e.sender().clone(),
            e.kind(),
            Some(e.state_key()).as_deref(),
            e.content().clone(),
            &auth_events,
            prev_events,
        );

        // This can be used to sort of test this function
        // match StateResolution::apply_event(
        //     &room_id(),
        //     &RoomVersionId::Version6,
        //     Arc::clone(&event),
        //     &state_after,
        //     Some(event_map.clone()),
        //     &store,
        // ) {
        //     Ok(res) => {
        //         println!(
        //             "res contains: {} passed: {} for {}\n{:?}",
        //             state_after
        //                 .get(&(event.kind(), event.state_key()))
        //                 .map(|id| id == &ev_id)
        //                 .unwrap_or_default(),
        //             res,
        //             event.event_id().as_str(),
        //             event
        //                 .prev_event_ids()
        //                 .iter()
        //                 .map(|id| id.to_string())
        //                 .collect::<Vec<_>>()
        //         );
        //     }
        //     Err(e) => panic!("resolution for {} failed: {}", node, e),
        // }

        // we have to update our store, an actual user of this lib would
        // be giving us state from a DB.
        store.0.insert(ev_id.clone(), Arc::clone(&event));

        state_at_event.insert(node, state_after);
        event_map.insert(event_id.clone(), event);
    }

    let mut expected_state = StateMap::new();
    for node in expected_state_ids {
        let ev = event_map.get(&node).unwrap_or_else(|| {
            panic!(
                "{} not found in {:?}",
                node.to_string(),
                event_map
                    .keys()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            )
        });

        let key = (ev.kind(), ev.state_key());

        expected_state.insert(key, node);
    }

    let start_state = state_at_event.get(&event_id("$START:foo")).unwrap();

    let end_state = state_at_event
        .get(&event_id("$END:foo"))
        .unwrap()
        .iter()
        .filter(|(k, v)| expected_state.contains_key(k) || start_state.get(k) != Some(*v))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<StateMap<EventId>>();

    assert_eq!(expected_state, end_state);
}

#[test]
fn ban_vs_power_level() {
    let events = &[
        to_init_pdu_event(
            "PA",
            alice(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({"users": {alice(): 100, bob(): 50}}),
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
            json!({"users": {alice(): 100, bob(): 50}}),
        ),
    ];

    let edges = vec![
        vec!["END", "MB", "MA", "PA", "START"],
        vec!["END", "PA", "PB"],
    ]
    .into_iter()
    .map(|list| list.into_iter().map(event_id).collect::<Vec<_>>())
    .collect::<Vec<_>>();

    let expected_state_ids = vec!["PA", "MA", "MB", "END"]
        .into_iter()
        .map(event_id)
        .collect::<Vec<_>>();

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
            json!({"users": {alice(): 100, bob(): 50}}),
        ),
        to_init_pdu_event("T2", alice(), EventType::RoomTopic, Some(""), json!({})),
        to_init_pdu_event(
            "PA2",
            alice(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({"users": {alice(): 100, bob(): 0}}),
        ),
        to_init_pdu_event(
            "PB",
            bob(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({"users": {alice(): 100, bob(): 50}}),
        ),
        to_init_pdu_event("T3", bob(), EventType::RoomTopic, Some(""), json!({})),
    ];

    let edges = vec![
        vec!["END", "PA2", "T2", "PA1", "T1", "START"],
        vec!["END", "T3", "PB", "PA1"],
    ]
    .into_iter()
    .map(|list| list.into_iter().map(event_id).collect::<Vec<_>>())
    .collect::<Vec<_>>();

    let expected_state_ids = vec!["PA2", "T2", "END"]
        .into_iter()
        .map(event_id)
        .collect::<Vec<_>>();

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
            json!({"users": {alice(): 100, bob(): 50}}),
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

    let edges = vec![
        vec!["END", "MB", "T2", "PA", "T1", "START"],
        vec!["END", "T1"],
    ]
    .into_iter()
    .map(|list| list.into_iter().map(event_id).collect::<Vec<_>>())
    .collect::<Vec<_>>();

    let expected_state_ids = vec!["T1", "MB", "PA", "END"]
        .into_iter()
        .map(event_id)
        .collect::<Vec<_>>();

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

    let expected_state_ids = vec![event_id("JR"), event_id("END")];

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
            json!({"users": {alice(): 100, bob(): 50}}),
        ),
        to_init_pdu_event(
            "PB",
            bob(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({"users": {alice(): 100, bob(): 50, charlie(): 50}}),
        ),
        to_init_pdu_event(
            "PC",
            charlie(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({"users": {alice(): 100, bob(): 50, charlie(): 0}}),
        ),
    ];

    let edges = vec![vec!["END", "PC", "PB", "PA", "START"], vec!["END", "PA"]]
        .into_iter()
        .map(|list| list.into_iter().map(event_id).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let expected_state_ids = vec!["PC", "END"]
        .into_iter()
        .map(event_id)
        .collect::<Vec<_>>();

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
            json!({"users": {alice(): 100, bob(): 50}}),
        ),
        to_init_pdu_event("T2", alice(), EventType::RoomTopic, Some(""), json!({})),
        to_init_pdu_event(
            "PA2",
            alice(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({"users": {alice(): 100, bob(): 0}}),
        ),
        to_init_pdu_event(
            "PB",
            bob(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({"users": {alice(): 100, bob(): 50}}),
        ),
        to_init_pdu_event("T3", bob(), EventType::RoomTopic, Some(""), json!({})),
        to_init_pdu_event("MZ1", zera(), EventType::RoomTopic, Some(""), json!({})),
        to_init_pdu_event("T4", alice(), EventType::RoomTopic, Some(""), json!({})),
    ];

    let edges = vec![
        vec!["END", "T4", "MZ1", "PA2", "T2", "PA1", "T1", "START"],
        vec!["END", "MZ1", "T3", "PB", "PA1"],
    ]
    .into_iter()
    .map(|list| list.into_iter().map(event_id).collect::<Vec<_>>())
    .collect::<Vec<_>>();

    let expected_state_ids = vec!["T4", "PA2", "END"]
        .into_iter()
        .map(event_id)
        .collect::<Vec<_>>();

    do_check(events, edges, expected_state_ids)
}

#[test]
fn test_event_map_none() {
    let mut store = TestStore(btreemap! {});

    // build up the DAG
    let (state_at_bob, state_at_charlie, expected) = store.set_up();

    let resolved = match StateResolution::resolve(
        &room_id(),
        &RoomVersionId::Version2,
        &[state_at_bob, state_at_charlie],
        None,
        &store,
    ) {
        Ok(state) => state,
        Err(e) => panic!("{}", e),
    };

    assert_eq!(expected, resolved)
}

#[test]
fn test_lexicographical_sort() {
    let graph = btreemap! {
        event_id("l") => vec![event_id("o")],
        event_id("m") => vec![event_id("n"), event_id("o")],
        event_id("n") => vec![event_id("o")],
        event_id("o") => vec![], // "o" has zero outgoing edges but 4 incoming edges
        event_id("p") => vec![event_id("o")],
    };

    let res =
        StateResolution::lexicographical_topological_sort(&graph, |id| (0, UNIX_EPOCH, id.clone()));

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

/// The test state store.
pub struct TestStore(BTreeMap<EventId, Arc<StateEvent>>);

#[allow(unused)]
impl StateStore for TestStore {
    fn get_event(&self, room_id: &RoomId, event_id: &EventId) -> Result<Arc<StateEvent>> {
        self.0
            .get(event_id)
            .map(Arc::clone)
            .ok_or_else(|| Error::NotFound(format!("{} not found", event_id.to_string())))
    }
}

impl TestStore {
    pub fn set_up(&mut self) -> (StateMap<EventId>, StateMap<EventId>, StateMap<EventId>) {
        // to activate logging use `RUST_LOG=debug cargo t one_test_only`
        let _ = LOGGER.call_once(|| {
            tracer::fmt()
                .with_env_filter(tracer::EnvFilter::from_default_env())
                .init()
        });
        let create_event = to_pdu_event::<EventId>(
            "CREATE",
            alice(),
            EventType::RoomCreate,
            Some(""),
            json!({ "creator": alice() }),
            &[],
            &[],
        );
        let cre = create_event.event_id();
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
        self.0.insert(alice_mem.event_id(), Arc::clone(&alice_mem));

        let join_rules = to_pdu_event(
            "IJR",
            alice(),
            EventType::RoomJoinRules,
            Some(""),
            json!({ "join_rule": JoinRule::Public }),
            &[cre.clone(), alice_mem.event_id()],
            &[alice_mem.event_id()],
        );
        self.0.insert(join_rules.event_id(), join_rules.clone());

        // Bob and Charlie join at the same time, so there is a fork
        // this will be represented in the state_sets when we resolve
        let bob_mem = to_pdu_event(
            "IMB",
            bob(),
            EventType::RoomMember,
            Some(bob().to_string().as_str()),
            member_content_join(),
            &[cre.clone(), join_rules.event_id()],
            &[join_rules.event_id()],
        );
        self.0.insert(bob_mem.event_id(), bob_mem.clone());

        let charlie_mem = to_pdu_event(
            "IMC",
            charlie(),
            EventType::RoomMember,
            Some(charlie().to_string().as_str()),
            member_content_join(),
            &[cre, join_rules.event_id()],
            &[join_rules.event_id()],
        );
        self.0.insert(charlie_mem.event_id(), charlie_mem.clone());

        let state_at_bob = [&create_event, &alice_mem, &join_rules, &bob_mem]
            .iter()
            .map(|e| ((e.kind(), e.state_key()), e.event_id()))
            .collect::<StateMap<_>>();

        let state_at_charlie = [&create_event, &alice_mem, &join_rules, &charlie_mem]
            .iter()
            .map(|e| ((e.kind(), e.state_key()), e.event_id()))
            .collect::<StateMap<_>>();

        let expected = [
            &create_event,
            &alice_mem,
            &join_rules,
            &bob_mem,
            &charlie_mem,
        ]
        .iter()
        .map(|e| ((e.kind(), e.state_key()), e.event_id()))
        .collect::<StateMap<_>>();

        (state_at_bob, state_at_charlie, expected)
    }
}
