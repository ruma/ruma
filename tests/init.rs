#![allow(unused)]

use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    convert::TryFrom,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use maplit::btreemap;
use ruma::{
    events::{
        pdu::Pdu,
        room::{
            join_rules::JoinRule,
            member::{MemberEventContent, MembershipState},
        },
        EventType,
    },
    identifiers::{EventId, RoomId, RoomVersionId, UserId},
};
use serde_json::{from_value as from_json_value, json, Value as JsonValue};
use state_res::{ResolutionResult, StateEvent, StateMap, StateResolution, StateStore};

static mut SERVER_TIMESTAMP: i32 = 0;

fn id(id: &str) -> EventId {
    EventId::try_from(format!("${}:foo", id)).unwrap()
}

fn alice() -> UserId {
    UserId::try_from("@alice:foo").unwrap()
}
fn bobo() -> UserId {
    UserId::try_from("@bobo:foo").unwrap()
}
fn devin() -> UserId {
    UserId::try_from("@devin:foo").unwrap()
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

fn to_pdu_event(
    id: &str,
    sender: UserId,
    ev_type: EventType,
    state_key: Option<&str>,
    content: JsonValue,
    auth_events: &[EventId],
    prev_events: &[EventId],
) -> StateEvent {
    let ts = unsafe {
        let ts = SERVER_TIMESTAMP;
        // increment the "origin_server_ts" value
        SERVER_TIMESTAMP += 1;
        ts
    };
    let id = if id.contains("$") {
        id.to_string()
    } else {
        format!("${}:foo", id)
    };

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
    serde_json::from_value(json).unwrap()
}

fn to_init_pdu_event(
    id: &str,
    sender: UserId,
    ev_type: EventType,
    state_key: Option<&str>,
    content: JsonValue,
) -> StateEvent {
    let ts = unsafe {
        let ts = SERVER_TIMESTAMP;
        // increment the "origin_server_ts" value
        SERVER_TIMESTAMP += 1;
        ts
    };
    let id = if id.contains("$") {
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
    serde_json::from_value(json).unwrap()
}

// all graphs start with these input events
#[allow(non_snake_case)]
fn INITIAL_EVENTS() -> BTreeMap<EventId, StateEvent> {
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
            bobo(),
            EventType::RoomMember,
            Some(bobo().to_string().as_str()),
            member_content_join(),
        ),
        to_init_pdu_event(
            "IMC",
            devin(),
            EventType::RoomMember,
            Some(devin().to_string().as_str()),
            member_content_join(),
        ),
        to_init_pdu_event(
            "IMZ",
            zera(),
            EventType::RoomMember,
            Some(zera().to_string().as_str()),
            member_content_join(),
        ),
        to_init_pdu_event("START", zera(), EventType::RoomMessage, None, json!({})),
        to_init_pdu_event("END", zera(), EventType::RoomMessage, None, json!({})),
    ]
    .into_iter()
    .map(|ev| (ev.event_id().unwrap().clone(), ev))
    .collect()
}

#[allow(non_snake_case)]
fn INITIAL_EDGES() -> Vec<EventId> {
    vec![
        "START", "IMZ", "IMC", "IMB", "IJR", "IPOWER", "IMA", "CREATE",
    ]
    .into_iter()
    .map(|s| format!("${}:foo", s))
    .map(EventId::try_from)
    .collect::<Result<Vec<_>, _>>()
    .unwrap()
}

pub struct TestStore(RefCell<BTreeMap<EventId, StateEvent>>);

#[allow(unused)]
impl StateStore for TestStore {
    fn get_events(&self, events: &[EventId]) -> Result<Vec<StateEvent>, String> {
        Ok(self
            .0
            .borrow()
            .iter()
            .filter(|e| events.contains(e.0))
            .map(|(_, s)| s)
            .cloned()
            .collect())
    }

    fn get_event(&self, event_id: &EventId) -> Result<StateEvent, String> {
        self.0
            .borrow()
            .get(event_id)
            .cloned()
            .ok_or(format!("{} not found", event_id.to_string()))
    }

    fn auth_event_ids(
        &self,
        room_id: &RoomId,
        event_ids: &[EventId],
    ) -> Result<Vec<EventId>, String> {
        let mut result = vec![];
        let mut stack = event_ids.to_vec();

        while !stack.is_empty() {
            let ev_id = stack.pop().unwrap();
            if result.contains(&ev_id) {
                continue;
            }

            result.push(ev_id.clone());

            let event = self.get_event(&ev_id).unwrap();
            for aid in event.auth_event_ids() {
                stack.push(aid);
            }
        }

        Ok(result)
    }

    fn auth_chain_diff(
        &self,
        room_id: &RoomId,
        event_ids: Vec<Vec<EventId>>,
    ) -> Result<Vec<EventId>, String> {
        use itertools::Itertools;

        println!(
            "EVENTS FOR AUTH {:?}",
            event_ids
                .iter()
                .map(|v| v.iter().map(ToString::to_string).collect::<Vec<_>>())
                .collect::<Vec<_>>()
        );

        let mut chains = vec![];
        for ids in event_ids {
            let chain = self
                .auth_event_ids(room_id, &ids)?
                .into_iter()
                .collect::<BTreeSet<_>>();
            chains.push(chain);
        }

        if let Some(chain) = chains.first() {
            let rest = chains.iter().skip(1).flatten().cloned().collect();
            let common = chain.intersection(&rest).collect::<Vec<_>>();
            println!(
                "COMMON {:?}",
                common.iter().map(ToString::to_string).collect::<Vec<_>>()
            );
            Ok(chains
                .iter()
                .flatten()
                .filter(|id| !common.contains(&id))
                .cloned()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect())
        } else {
            Ok(vec![])
        }
    }
}

fn do_check(events: &[StateEvent], edges: Vec<Vec<EventId>>, expected_state_ids: Vec<EventId>) {
    use itertools::Itertools;

    let mut resolver = StateResolution::default();
    // TODO what do we fill this with, everything ??
    let store = TestStore(RefCell::new(
        INITIAL_EVENTS()
            .values()
            .chain(events)
            .map(|ev| (ev.event_id().unwrap().clone(), ev.clone()))
            .collect(),
    ));

    // This will be lexi_topo_sorted for resolution
    let mut graph = BTreeMap::new();
    // this is the same as in `resolve` event_id -> StateEvent
    let mut fake_event_map = BTreeMap::new();

    // create the DB of events that led up to this point
    // TODO maybe clean up some of these clones it is just tests but...
    for ev in INITIAL_EVENTS().values().chain(events) {
        println!("{:?}", ev.event_id().unwrap().to_string());
        graph.insert(ev.event_id().unwrap().clone(), vec![]);
        fake_event_map.insert(ev.event_id().unwrap().clone(), ev.clone());
    }

    for pair in INITIAL_EDGES().windows(2) {
        if let &[a, b] = &pair {
            graph.entry(a.clone()).or_insert(vec![]).push(b.clone());
        }
    }

    for edge_list in edges {
        for pair in edge_list.windows(2) {
            if let &[a, b] = &pair {
                graph.entry(a.clone()).or_insert(vec![]).push(b.clone());
            }
        }
    }

    // event_id -> StateEvent
    let mut event_map: BTreeMap<EventId, StateEvent> = BTreeMap::new();
    // event_id -> StateMap<EventId>
    let mut state_at_event: BTreeMap<EventId, StateMap<EventId>> = BTreeMap::new();

    // resolve the current state and add it to the state_at_event map then continue
    // on in "time"?
    for node in resolver
        // TODO is this `key_fn` return correct ??
        .lexicographical_topological_sort(&graph, |id| (0, UNIX_EPOCH, Some(id.clone())))
    {
        println!("{}", node.to_string());
        let fake_event = fake_event_map.get(&node).unwrap();
        let event_id = fake_event.event_id().unwrap();

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

            // println!(
            //     "resolving {:#?}",
            //     state_sets
            //         .iter()
            //         .map(|map| map
            //             .iter()
            //             .map(|((t, s), id)| (t, s, id.to_string()))
            //             .collect::<Vec<_>>())
            //         .collect::<Vec<_>>()
            // );

            let resolved = resolver.resolve(
                &room_id(),
                &RoomVersionId::version_1(),
                &state_sets,
                Some(event_map.clone()),
                &store,
            );
            match resolved {
                Ok(ResolutionResult::Resolved(state)) => state,
                _ => panic!("resolution for {} failed", node),
            }
        };

        let mut state_after = state_before.clone();
        if fake_event.state_key().is_some() {
            let ty = fake_event.kind().clone();
            // we know there is a state_key unwrap OK
            let key = fake_event.state_key().unwrap().clone();
            state_after.insert((ty, key), event_id.clone());
        }

        let auth_types = state_res::auth_types_for_event(fake_event);
        println!(
            "AUTH TYPES {:?}",
            auth_types.iter().map(|(t, id)| (t, id)).collect::<Vec<_>>()
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
        let ev_id = e.event_id().unwrap();
        let event = to_pdu_event(
            &e.event_id().unwrap().to_string(),
            e.sender().clone(),
            e.kind(),
            e.state_key().as_deref(),
            e.content(),
            &auth_events,
            prev_events,
        );
        // we have to update our store, an actual user of this lib would do this
        // with the result of the resolution>
        *store.0.borrow_mut().get_mut(ev_id).unwrap() = event.clone();

        state_at_event.insert(node, state_after);
        event_map.insert(event_id.clone(), event);
    }

    let mut expected_state = BTreeMap::new();
    for node in expected_state_ids {
        let ev = event_map.get(&node).expect(&format!(
            "{} not found in {:?}",
            node.to_string(),
            event_map
                .keys()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
        ));

        let key = (ev.kind(), ev.state_key().unwrap());

        expected_state.insert(key, node);
    }

    let start_state = state_at_event
        .get(&EventId::try_from("$START:foo").unwrap())
        .unwrap();

    let end_state = state_at_event
        .get(&EventId::try_from("$END:foo").unwrap())
        .unwrap()
        .iter()
        .filter(|(k, v)| {
            println!(
                "{:?} == {:?}",
                start_state.get(k).map(ToString::to_string),
                Some(v.to_string())
            );
            expected_state.contains_key(k) || start_state.get(k) != Some(*v)
        })
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
            json!({"users": {alice(): 100, bobo(): 50}}),
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
            Some(bobo().to_string().as_str()),
            member_content_ban(),
        ),
        to_init_pdu_event(
            "PB",
            bobo(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({"users": {alice(): 100, bobo(): 50}}),
        ),
    ];

    let edges = vec![
        vec!["END", "MB", "MA", "PA", "START"],
        vec!["END", "PB", "PA"],
    ]
    .into_iter()
    .map(|list| {
        list.into_iter()
            .map(|s| format!("${}:foo", s))
            .map(EventId::try_from)
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    })
    .collect::<Vec<_>>();

    let expected_state_ids = vec!["PA", "MA", "MB"]
        .into_iter()
        .map(|s| format!("${}:foo", s))
        .map(EventId::try_from)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    do_check(events, edges, expected_state_ids)
}

// #[test]
fn topic_reset() {
    let events = &[
        to_init_pdu_event("T1", alice(), EventType::RoomTopic, Some(""), json!({})),
        to_init_pdu_event(
            "PA",
            alice(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({"users": {alice(): 100, bobo(): 50}}),
        ),
        to_init_pdu_event("T2", bobo(), EventType::RoomTopic, Some(""), json!({})),
        to_init_pdu_event(
            "MB",
            alice(),
            EventType::RoomMember,
            Some(bobo().to_string().as_str()),
            member_content_ban(),
        ),
    ];

    let edges = vec![
        vec!["END", "MB", "T2", "PA", "T1", "START"],
        vec!["END", "T1"],
    ]
    .into_iter()
    .map(|list| {
        list.into_iter()
            .map(|s| format!("${}:foo", s))
            .map(EventId::try_from)
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    })
    .collect::<Vec<_>>();

    let expected_state_ids = vec!["T1", "MB", "PA"]
        .into_iter()
        .map(|s| format!("${}:foo", s))
        .map(EventId::try_from)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    do_check(events, edges, expected_state_ids)
}

#[test]
fn test_lexicographical_sort() {
    let mut resolver = StateResolution::default();

    let graph = btreemap! {
        id("l") => vec![id("o")],
        id("m") => vec![id("n"), id("o")],
        id("n") => vec![id("o")],
        id("o") => vec![], // "o" has zero outgoing edges but 4 incoming edges
        id("p") => vec![id("o")],
    };

    let res =
        resolver.lexicographical_topological_sort(&graph, |id| (0, UNIX_EPOCH, Some(id.clone())));

    assert_eq!(
        vec!["o", "l", "n", "m", "p"],
        res.iter()
            .map(ToString::to_string)
            .map(|s| s.replace("$", "").replace(":foo", ""))
            .collect::<Vec<_>>()
    )
}
