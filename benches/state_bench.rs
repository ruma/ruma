// `cargo bench` works, but if you use `cargo bench -- --save-baseline <name>`
// or pass any other args to it, it fails with the error
// `cargo bench unknown option --save-baseline`.
// To pass args to criterion, use this form
// `cargo bench --bench <name of the bench> -- --save-baseline <name>`.
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    convert::TryFrom,
    time::UNIX_EPOCH,
};

use criterion::{criterion_group, criterion_main, Criterion};
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
use state_res::{ResolutionResult, StateEvent, StateMap, StateResolution, StateStore};

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
            let mut resolver = StateResolution::default();

            let _ = resolver
                .lexicographical_topological_sort(&graph, |id| (0, UNIX_EPOCH, Some(id.clone())));
        })
    });
}

fn resolution_shallow_auth_chain(c: &mut Criterion) {
    c.bench_function("resolve state of 5 events one fork", |b| {
        let mut resolver = StateResolution::default();

        let store = TestStore(RefCell::new(btreemap! {}));

        // build up the DAG
        let (state_at_bob, state_at_charlie, _) = store.set_up();

        b.iter(|| {
            let _resolved = match resolver.resolve(
                &room_id(),
                &RoomVersionId::version_2(),
                &[state_at_bob.clone(), state_at_charlie.clone()],
                None,
                &store,
            ) {
                Ok(ResolutionResult::Resolved(state)) => state,
                Err(e) => panic!("{}", e),
                _ => panic!("conflicted state left"),
            };
        })
    });
}

criterion_group!(benches, lexico_topo_sort, resolution_shallow_auth_chain);

criterion_main!(benches);

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

        // DFS for auth event chain
        while !stack.is_empty() {
            let ev_id = stack.pop().unwrap();
            if result.contains(&ev_id) {
                continue;
            }

            result.push(ev_id.clone());

            let event = self.get_event(&ev_id).unwrap();
            stack.extend(event.auth_event_ids());
        }

        Ok(result)
    }

    fn auth_chain_diff(
        &self,
        room_id: &RoomId,
        event_ids: Vec<Vec<EventId>>,
    ) -> Result<Vec<EventId>, String> {
        use itertools::Itertools;

        let mut chains = vec![];
        for ids in event_ids {
            // TODO state store `auth_event_ids` returns self in the event ids list
            // when an event returns `auth_event_ids` self is not contained
            let chain = self
                .auth_event_ids(room_id, &ids)?
                .into_iter()
                .collect::<BTreeSet<_>>();
            chains.push(chain);
        }

        if let Some(chain) = chains.first() {
            let rest = chains.iter().skip(1).flatten().cloned().collect();
            let common = chain.intersection(&rest).collect::<Vec<_>>();

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

impl TestStore {
    pub fn set_up(&self) -> (StateMap<EventId>, StateMap<EventId>, StateMap<EventId>) {
        let create_event = to_pdu_event::<EventId>(
            "CREATE",
            alice(),
            EventType::RoomCreate,
            Some(""),
            json!({ "creator": alice() }),
            &[],
            &[],
        );
        let cre = create_event.event_id().unwrap().clone();
        self.0
            .borrow_mut()
            .insert(cre.clone(), create_event.clone());

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
            .borrow_mut()
            .insert(alice_mem.event_id().unwrap().clone(), alice_mem.clone());

        let join_rules = to_pdu_event(
            "IJR",
            alice(),
            EventType::RoomJoinRules,
            Some(""),
            json!({ "join_rule": JoinRule::Public }),
            &[cre.clone(), alice_mem.event_id().unwrap().clone()],
            &[alice_mem.event_id().unwrap().clone()],
        );
        self.0
            .borrow_mut()
            .insert(join_rules.event_id().unwrap().clone(), join_rules.clone());

        // Bob and Charlie join at the same time, so there is a fork
        // this will be represented in the state_sets when we resolve
        let bob_mem = to_pdu_event(
            "IMB",
            bob(),
            EventType::RoomMember,
            Some(bob().to_string().as_str()),
            member_content_join(),
            &[cre.clone(), join_rules.event_id().unwrap().clone()],
            &[join_rules.event_id().unwrap().clone()],
        );
        self.0
            .borrow_mut()
            .insert(bob_mem.event_id().unwrap().clone(), bob_mem.clone());

        let charlie_mem = to_pdu_event(
            "IMC",
            charlie(),
            EventType::RoomMember,
            Some(charlie().to_string().as_str()),
            member_content_join(),
            &[cre.clone(), join_rules.event_id().unwrap().clone()],
            &[join_rules.event_id().unwrap().clone()],
        );
        self.0
            .borrow_mut()
            .insert(charlie_mem.event_id().unwrap().clone(), charlie_mem.clone());

        let state_at_bob = [&create_event, &alice_mem, &join_rules, &bob_mem]
            .iter()
            .map(|e| {
                (
                    (e.kind(), e.state_key().unwrap().clone()),
                    e.event_id().unwrap().clone(),
                )
            })
            .collect::<BTreeMap<_, _>>();

        let state_at_charlie = [&create_event, &alice_mem, &join_rules, &charlie_mem]
            .iter()
            .map(|e| {
                (
                    (e.kind(), e.state_key().unwrap().clone()),
                    e.event_id().unwrap().clone(),
                )
            })
            .collect::<BTreeMap<_, _>>();

        let expected = [
            &create_event,
            &alice_mem,
            &join_rules,
            &bob_mem,
            &charlie_mem,
        ]
        .iter()
        .map(|e| {
            (
                (e.kind(), e.state_key().unwrap().clone()),
                e.event_id().unwrap().clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();

        (state_at_bob, state_at_charlie, expected)
    }
}

fn event_id(id: &str) -> EventId {
    if id.contains("$") {
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

fn room_id() -> RoomId {
    RoomId::try_from("!test:foo").unwrap()
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
) -> StateEvent
where
    S: AsRef<str>,
{
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
    let auth_events = auth_events
        .iter()
        .map(AsRef::as_ref)
        .map(|s| {
            EventId::try_from(
                if s.contains("$") {
                    s.to_owned()
                } else {
                    format!("${}:foo", s)
                }
                .as_str(),
            )
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let prev_events = prev_events
        .iter()
        .map(AsRef::as_ref)
        .map(|s| {
            EventId::try_from(
                if s.contains("$") {
                    s.to_owned()
                } else {
                    format!("${}:foo", s)
                }
                .as_str(),
            )
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

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
