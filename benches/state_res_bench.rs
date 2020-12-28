// Because of criterion `cargo bench` works,
// but if you use `cargo bench -- --save-baseline <name>`
// or pass any other args to it, it fails with the error
// `cargo bench unknown option --save-baseline`.
// To pass args to criterion, use this form
// `cargo bench --bench <name of the bench> -- --save-baseline <name>`.
use std::{collections::BTreeMap, convert::TryFrom, sync::Arc, time::UNIX_EPOCH};

use criterion::{criterion_group, criterion_main, Criterion};
use maplit::btreemap;
use ruma::{
    events::{
        pdu::ServerPdu,
        room::{
            join_rules::JoinRule,
            member::{MemberEventContent, MembershipState},
        },
        EventType,
    },
    identifiers::{EventId, RoomId, RoomVersionId, UserId},
};
use serde_json::{json, Value as JsonValue};
use state_res::{Error, Result, StateMap, StateResolution, StateStore};

static mut SERVER_TIMESTAMP: i32 = 0;

fn lexico_topo_sort(c: &mut Criterion) {
    c.bench_function("lexicographical topological sort", |b| {
        let graph = btreemap! {
            event_id("l") => vec![event_id("o")],
            event_id("m") => vec![event_id("n"), event_id("o")],
            event_id("n") => vec![event_id("o")],
            event_id("o") => vec![], // "o" has zero outgoing edges but 4 incoming edges
            event_id("p") => vec![event_id("o")],
        };
        b.iter(|| {
            let _ = StateResolution::lexicographical_topological_sort(&graph, |id| {
                (0, UNIX_EPOCH, id.clone())
            });
        })
    });
}

fn resolution_shallow_auth_chain(c: &mut Criterion) {
    c.bench_function("resolve state of 5 events one fork", |b| {
        let mut store = TestStore(btreemap! {});

        // build up the DAG
        let (state_at_bob, state_at_charlie, _) = store.set_up();

        b.iter(|| {
            let _resolved = match StateResolution::resolve(
                &room_id(),
                &RoomVersionId::Version6,
                &[state_at_bob.clone(), state_at_charlie.clone()],
                None,
                &store,
            ) {
                Ok(state) => state,
                Err(e) => panic!("{}", e),
            };
        })
    });
}

fn resolve_deeper_event_set(c: &mut Criterion) {
    c.bench_function("resolve state of 10 events 3 conflicting", |b| {
        let init = INITIAL_EVENTS();
        let ban = BAN_STATE_SET();

        let mut inner = init;
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
        .map(|ev| ((ev.kind.clone(), ev.state_key.clone()), ev.event_id.clone()))
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
        .map(|ev| ((ev.kind.clone(), ev.state_key.clone()), ev.event_id.clone()))
        .collect::<StateMap<_>>();

        b.iter(|| {
            let _resolved = match StateResolution::resolve(
                &room_id(),
                &RoomVersionId::Version2,
                &[state_set_a.clone(), state_set_b.clone()],
                Some(inner.clone()),
                &store,
            ) {
                Ok(state) => state,
                Err(_) => panic!("resolution failed during benchmarking"),
            };
        })
    });
}

criterion_group!(
    benches,
    // lexico_topo_sort,
    resolution_shallow_auth_chain,
    resolve_deeper_event_set
);

criterion_main!(benches);

//*/////////////////////////////////////////////////////////////////////
//
//  IMPLEMENTATION DETAILS AHEAD
//
/////////////////////////////////////////////////////////////////////*/
pub struct TestStore(BTreeMap<EventId, Arc<ServerPdu>>);

#[allow(unused)]
impl StateStore for TestStore {
    fn get_event(&self, room_id: &RoomId, event_id: &EventId) -> Result<Arc<ServerPdu>> {
        self.0
            .get(event_id)
            .map(Arc::clone)
            .ok_or_else(|| Error::NotFound(format!("{} not found", event_id.to_string())))
    }
}

impl TestStore {
    pub fn set_up(&mut self) -> (StateMap<EventId>, StateMap<EventId>, StateMap<EventId>) {
        let create_event = Arc::new(to_pdu_event::<EventId>(
            "CREATE",
            alice(),
            EventType::RoomCreate,
            Some(""),
            json!({ "creator": alice() }),
            &[],
            &[],
        ));
        let cre = create_event.event_id.clone();
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
        self.0
            .insert(alice_mem.event_id.clone(), Arc::clone(&alice_mem));

        let join_rules = to_pdu_event(
            "IJR",
            alice(),
            EventType::RoomJoinRules,
            Some(""),
            json!({ "join_rule": JoinRule::Public }),
            &[cre.clone(), alice_mem.event_id.clone()],
            &[alice_mem.event_id.clone()],
        );
        self.0
            .insert(join_rules.event_id.clone(), Arc::clone(&join_rules));

        // Bob and Charlie join at the same time, so there is a fork
        // this will be represented in the state_sets when we resolve
        let bob_mem = to_pdu_event(
            "IMB",
            bob(),
            EventType::RoomMember,
            Some(bob().to_string().as_str()),
            member_content_join(),
            &[cre.clone(), join_rules.event_id.clone()],
            &[join_rules.event_id.clone()],
        );
        self.0
            .insert(bob_mem.event_id.clone(), Arc::clone(&bob_mem));

        let charlie_mem = to_pdu_event(
            "IMC",
            charlie(),
            EventType::RoomMember,
            Some(charlie().to_string().as_str()),
            member_content_join(),
            &[cre, join_rules.event_id.clone()],
            &[join_rules.event_id.clone()],
        );
        self.0
            .insert(charlie_mem.event_id.clone(), Arc::clone(&charlie_mem));

        let state_at_bob = [&create_event, &alice_mem, &join_rules, &bob_mem]
            .iter()
            .map(|e| ((e.kind.clone(), e.state_key.clone()), e.event_id.clone()))
            .collect::<StateMap<_>>();

        let state_at_charlie = [&create_event, &alice_mem, &join_rules, &charlie_mem]
            .iter()
            .map(|e| ((e.kind.clone(), e.state_key.clone()), e.event_id.clone()))
            .collect::<StateMap<_>>();

        let expected = [
            &create_event,
            &alice_mem,
            &join_rules,
            &bob_mem,
            &charlie_mem,
        ]
        .iter()
        .map(|e| ((e.kind.clone(), e.state_key.clone()), e.event_id.clone()))
        .collect::<StateMap<_>>();

        (state_at_bob, state_at_charlie, expected)
    }
}

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
) -> Arc<ServerPdu>
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

// all graphs start with these input events
#[allow(non_snake_case)]
fn INITIAL_EVENTS() -> BTreeMap<EventId, Arc<ServerPdu>> {
    vec![
        to_pdu_event::<EventId>(
            "CREATE",
            alice(),
            EventType::RoomCreate,
            Some(""),
            json!({ "creator": alice() }),
            &[],
            &[],
        ),
        to_pdu_event(
            "IMA",
            alice(),
            EventType::RoomMember,
            Some(alice().to_string().as_str()),
            member_content_join(),
            &["CREATE"],
            &["CREATE"],
        ),
        to_pdu_event(
            "IPOWER",
            alice(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({"users": {alice().to_string(): 100}}),
            &["CREATE", "IMA"],
            &["IMA"],
        ),
        to_pdu_event(
            "IJR",
            alice(),
            EventType::RoomJoinRules,
            Some(""),
            json!({ "join_rule": JoinRule::Public }),
            &["CREATE", "IMA", "IPOWER"],
            &["IPOWER"],
        ),
        to_pdu_event(
            "IMB",
            bob(),
            EventType::RoomMember,
            Some(bob().to_string().as_str()),
            member_content_join(),
            &["CREATE", "IJR", "IPOWER"],
            &["IJR"],
        ),
        to_pdu_event(
            "IMC",
            charlie(),
            EventType::RoomMember,
            Some(charlie().to_string().as_str()),
            member_content_join(),
            &["CREATE", "IJR", "IPOWER"],
            &["IMB"],
        ),
        to_pdu_event::<EventId>(
            "START",
            charlie(),
            EventType::RoomTopic,
            Some(""),
            json!({}),
            &[],
            &[],
        ),
        to_pdu_event::<EventId>(
            "END",
            charlie(),
            EventType::RoomTopic,
            Some(""),
            json!({}),
            &[],
            &[],
        ),
    ]
    .into_iter()
    .map(|ev| (ev.event_id.clone(), ev))
    .collect()
}

// all graphs start with these input events
#[allow(non_snake_case)]
fn BAN_STATE_SET() -> BTreeMap<EventId, Arc<ServerPdu>> {
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
    .map(|ev| (ev.event_id.clone(), ev))
    .collect()
}
