use std::{
    borrow::Borrow,
    collections::{BTreeMap, HashMap, HashSet},
    slice,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering::SeqCst},
    },
};

use js_int::{int, uint};
use ruma_common::{
    EventId, MilliSecondsSinceUnixEpoch, RoomId, ServerSignatures, UserId, event_id, room_id,
    room_version_rules::{AuthorizationRules, StateResolutionV2Rules},
    user_id,
};
use ruma_events::{
    StateEventType, TimelineEventType,
    room::{
        join_rules::{JoinRule, RoomJoinRulesEventContent},
        member::{MembershipState, RoomMemberEventContent},
    },
};
use serde_json::{
    json,
    value::{RawValue as RawJsonValue, to_raw_value as to_raw_json_value},
};
use tracing::info;

pub(crate) use self::event::{EventHash, PduEvent};
use crate::{
    Error, Event, Result, StateMap, auth_types_for_event, events::RoomCreateEvent,
    state_res::EventTypeExt,
};

static SERVER_TIMESTAMP: AtomicU64 = AtomicU64::new(0);

pub(crate) fn do_check(
    events: &[Arc<PduEvent>],
    edges: Vec<Vec<EventId>>,
    expected_state_ids: Vec<EventId>,
) {
    // To activate logging use `RUST_LOG=debug cargo t`

    let init_events = INITIAL_EVENTS();

    let mut store = TestStore(
        init_events
            .values()
            .chain(events)
            .map(|ev| (ev.event_id().to_owned(), ev.clone()))
            .collect(),
    );

    // This will be lexi_topo_sorted for resolution
    let mut graph = HashMap::new();
    // This is the same as in `resolve` event_id -> OriginalStateEvent
    let mut fake_event_map = HashMap::new();

    // Create the DB of events that led up to this point
    // TODO maybe clean up some of these clones it is just tests but...
    for ev in init_events.values().chain(events) {
        graph.insert(ev.event_id().to_owned(), HashSet::new());
        fake_event_map.insert(ev.event_id().to_owned(), ev.clone());
    }

    for pair in INITIAL_EDGES().windows(2) {
        if let [a, b] = &pair {
            graph.entry(a.to_owned()).or_insert_with(HashSet::new).insert(b.clone());
        }
    }

    for edge_list in edges {
        for pair in edge_list.windows(2) {
            if let [a, b] = &pair {
                graph.entry(a.to_owned()).or_insert_with(HashSet::new).insert(b.clone());
            }
        }
    }

    // event_id -> PduEvent
    let mut event_map: HashMap<EventId, Arc<PduEvent>> = HashMap::new();
    // event_id -> StateMap<EventId>
    let mut state_at_event: HashMap<EventId, StateMap<EventId>> = HashMap::new();

    // Resolve the current state and add it to the state_at_event map then continue
    // on in "time"
    for node in crate::reverse_topological_power_sort(&graph, |_id| {
        Ok((int!(0).into(), MilliSecondsSinceUnixEpoch(uint!(0))))
    })
    .unwrap()
    {
        let fake_event = fake_event_map.get(&node).unwrap();
        let event_id = fake_event.event_id().to_owned();

        let prev_events = graph.get(&node).unwrap();

        let state_before: StateMap<EventId> = if prev_events.is_empty() {
            HashMap::new()
        } else if prev_events.len() == 1 {
            state_at_event.get(prev_events.iter().next().unwrap()).unwrap().clone()
        } else {
            let state_sets =
                prev_events.iter().filter_map(|k| state_at_event.get(k)).collect::<Vec<_>>();

            info!(
                "{:#?}",
                state_sets
                    .iter()
                    .map(|map| map
                        .iter()
                        .map(|((ty, key), id)| format!("(({ty}{key:?}), {id})"))
                        .collect::<Vec<_>>())
                    .collect::<Vec<_>>()
            );

            let auth_chain_sets = state_sets
                .iter()
                .map(|map| {
                    store.auth_event_ids(room_id(), map.values().cloned().collect()).unwrap()
                })
                .collect();

            let resolved = crate::resolve(
                &AuthorizationRules::V6,
                &StateResolutionV2Rules::V2_0,
                state_sets,
                auth_chain_sets,
                |id| event_map.get(id).cloned(),
                |_| unreachable!(),
            );
            match resolved {
                Ok(state) => state,
                Err(e) => panic!("resolution for {node} failed: {e}"),
            }
        };

        let mut state_after = state_before.clone();

        let ty = fake_event.event_type();
        let key = fake_event.state_key().unwrap();
        state_after.insert(ty.with_state_key(key), event_id.to_owned());

        let auth_types = auth_types_for_event(
            fake_event.event_type(),
            fake_event.sender(),
            fake_event.state_key(),
            fake_event.content(),
            &AuthorizationRules::V6,
        )
        .unwrap();

        let mut auth_events = vec![];
        for key in auth_types {
            if state_before.contains_key(&key) {
                auth_events.push(state_before[&key].clone());
            }
        }

        // TODO The event is just remade, adding the auth_events and prev_events here
        // the `to_pdu_event` was split into `init` and the fn below, could be better
        let e = fake_event;
        let ev_id = e.event_id();
        let event = to_pdu_event(
            e.event_id().as_str(),
            e.sender(),
            e.event_type().clone(),
            e.state_key(),
            e.content().to_owned(),
            &auth_events,
            &prev_events.iter().cloned().collect::<Vec<_>>(),
        );

        // We have to update our store, an actual user of this lib would
        // be giving us state from a DB.
        store.0.insert(ev_id.to_owned(), event.clone());

        state_at_event.insert(node, state_after);
        event_map.insert(event_id.to_owned(), Arc::clone(store.0.get(ev_id).unwrap()));
    }

    let mut expected_state = StateMap::new();
    for node in expected_state_ids {
        let ev = event_map.get(&node).unwrap_or_else(|| {
            panic!(
                "{node} not found in {:?}",
                event_map.keys().map(ToString::to_string).collect::<Vec<_>>()
            )
        });

        let key = ev.event_type().with_state_key(ev.state_key().unwrap());

        expected_state.insert(key, node);
    }

    let start_state = state_at_event.get(event_id!("$START:foo")).unwrap();

    let end_state = state_at_event
        .get(event_id!("$END:foo"))
        .unwrap()
        .iter()
        .filter(|(k, v)| {
            expected_state.contains_key(k)
                || start_state.get(k) != Some(*v)
                // Filter out the dummy messages events.
                // These act as points in time where there should be a known state to
                // test against.
                && **k != ("m.room.message".into(), "dummy".to_owned())
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<StateMap<EventId>>();

    assert_eq!(expected_state, end_state);
}

#[allow(clippy::exhaustive_structs)]
pub(crate) struct TestStore<E: Event>(pub(crate) HashMap<EventId, Arc<E>>);

impl<E: Event> TestStore<E> {
    pub(crate) fn get_event(&self, _: &RoomId, event_id: &EventId) -> Result<Arc<E>> {
        self.0.get(event_id).cloned().ok_or_else(|| Error::NotFound(event_id.to_owned()))
    }

    /// Returns a Vec of the related auth events to the given `event`.
    pub(crate) fn auth_event_ids(
        &self,
        room_id: &RoomId,
        event_ids: Vec<E::Id>,
    ) -> Result<HashSet<E::Id>> {
        let mut result = HashSet::new();
        let mut stack = event_ids;

        // DFS for auth event chain
        while let Some(ev_id) = stack.pop() {
            if result.contains(&ev_id) {
                continue;
            }

            result.insert(ev_id.clone());

            let event = self.get_event(room_id, ev_id.borrow())?;

            stack.extend(event.auth_events().map(ToOwned::to_owned));
        }

        Ok(result)
    }
}

// A StateStore implementation for testing
#[allow(clippy::type_complexity)]
impl TestStore<PduEvent> {
    pub(crate) fn set_up(&mut self) -> (StateMap<EventId>, StateMap<EventId>, StateMap<EventId>) {
        let create_event = to_pdu_event::<&EventId>(
            "CREATE",
            alice(),
            TimelineEventType::RoomCreate,
            Some(""),
            to_raw_json_value(&json!({ "creator": alice() })).unwrap(),
            &[],
            &[],
        );
        let cre = create_event.event_id().to_owned();
        self.0.insert(cre.clone(), Arc::clone(&create_event));

        let alice_mem = to_pdu_event(
            "IMA",
            alice(),
            TimelineEventType::RoomMember,
            Some(alice().as_str()),
            member_content_join(),
            slice::from_ref(&cre),
            slice::from_ref(&cre),
        );
        self.0.insert(alice_mem.event_id().to_owned(), Arc::clone(&alice_mem));

        let join_rules = to_pdu_event(
            "IJR",
            alice(),
            TimelineEventType::RoomJoinRules,
            Some(""),
            to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Public)).unwrap(),
            &[cre.clone(), alice_mem.event_id().to_owned()],
            &[alice_mem.event_id().to_owned()],
        );
        self.0.insert(join_rules.event_id().to_owned(), join_rules.clone());

        // Bob and Charlie join at the same time, so there is a fork
        // this will be represented in the state_sets when we resolve
        let bob_mem = to_pdu_event(
            "IMB",
            bob(),
            TimelineEventType::RoomMember,
            Some(bob().as_str()),
            member_content_join(),
            &[cre.clone(), join_rules.event_id().to_owned()],
            &[join_rules.event_id().to_owned()],
        );
        self.0.insert(bob_mem.event_id().to_owned(), bob_mem.clone());

        let charlie_mem = to_pdu_event(
            "IMC",
            charlie(),
            TimelineEventType::RoomMember,
            Some(charlie().as_str()),
            member_content_join(),
            &[cre, join_rules.event_id().to_owned()],
            &[join_rules.event_id().to_owned()],
        );
        self.0.insert(charlie_mem.event_id().to_owned(), charlie_mem.clone());

        let state_at_bob = [&create_event, &alice_mem, &join_rules, &bob_mem]
            .iter()
            .map(|e| {
                (e.event_type().with_state_key(e.state_key().unwrap()), e.event_id().to_owned())
            })
            .collect::<StateMap<_>>();

        let state_at_charlie = [&create_event, &alice_mem, &join_rules, &charlie_mem]
            .iter()
            .map(|e| {
                (e.event_type().with_state_key(e.state_key().unwrap()), e.event_id().to_owned())
            })
            .collect::<StateMap<_>>();

        let expected = [&create_event, &alice_mem, &join_rules, &bob_mem, &charlie_mem]
            .iter()
            .map(|e| {
                (e.event_type().with_state_key(e.state_key().unwrap()), e.event_id().to_owned())
            })
            .collect::<StateMap<_>>();

        (state_at_bob, state_at_charlie, expected)
    }
}

pub(crate) fn event_id(id: &str) -> EventId {
    if id.contains('$') {
        return id.try_into().unwrap();
    }

    format!("${id}:foo").try_into().unwrap()
}

pub(crate) fn alice() -> &'static UserId {
    user_id!("@alice:foo")
}

pub(crate) fn bob() -> &'static UserId {
    user_id!("@bob:foo")
}

pub(crate) fn charlie() -> &'static UserId {
    user_id!("@charlie:foo")
}

pub(crate) fn ella() -> &'static UserId {
    user_id!("@ella:foo")
}

pub(crate) fn zara() -> &'static UserId {
    user_id!("@zara:foo")
}

pub(crate) fn room_id() -> &'static RoomId {
    room_id!("!test:foo")
}

pub(crate) fn v12_room_id() -> &'static RoomId {
    room_id!("!CREATE")
}

pub(crate) fn member_content_ban() -> Box<RawJsonValue> {
    to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Ban)).unwrap()
}

pub(crate) fn member_content_join() -> Box<RawJsonValue> {
    to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Join)).unwrap()
}

pub(crate) fn to_init_pdu_event(
    id: &str,
    sender: &UserId,
    ev_type: TimelineEventType,
    state_key: Option<&str>,
    content: Box<RawJsonValue>,
) -> Arc<PduEvent> {
    let ts = SERVER_TIMESTAMP.fetch_add(1, SeqCst);
    let id = if id.contains('$') { id.to_owned() } else { format!("${id}:foo") };

    let state_key = state_key.map(ToOwned::to_owned);
    Arc::new(PduEvent {
        event_id: id.try_into().unwrap(),
        room_id: Some(room_id().to_owned()),
        sender: sender.to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(ts.try_into().unwrap()),
        state_key,
        kind: ev_type,
        content,
        redacts: None,
        unsigned: BTreeMap::new(),
        auth_events: vec![],
        prev_events: vec![],
        depth: uint!(0),
        hashes: EventHash { sha256: "".to_owned() },
        signatures: ServerSignatures::default(),
        rejected: false,
    })
}

pub(crate) fn to_pdu_event<S>(
    id: &str,
    sender: &UserId,
    ev_type: TimelineEventType,
    state_key: Option<&str>,
    content: Box<RawJsonValue>,
    auth_events: &[S],
    prev_events: &[S],
) -> Arc<PduEvent>
where
    S: AsRef<str>,
{
    let ts = SERVER_TIMESTAMP.fetch_add(1, SeqCst);
    let id = if id.contains('$') { id.to_owned() } else { format!("${id}:foo") };
    let auth_events = auth_events.iter().map(AsRef::as_ref).map(event_id).collect::<Vec<_>>();
    let prev_events = prev_events.iter().map(AsRef::as_ref).map(event_id).collect::<Vec<_>>();

    let state_key = state_key.map(ToOwned::to_owned);
    Arc::new(PduEvent {
        event_id: id.try_into().unwrap(),
        room_id: Some(room_id().to_owned()),
        sender: sender.to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(ts.try_into().unwrap()),
        state_key,
        kind: ev_type,
        content,
        redacts: None,
        unsigned: BTreeMap::new(),
        auth_events,
        prev_events,
        depth: uint!(0),
        hashes: EventHash { sha256: "".to_owned() },
        signatures: ServerSignatures::default(),
        rejected: false,
    })
}

/// Same as `to_pdu_event()`, but uses the default m.room.create event ID to generate the room ID.
pub(crate) fn to_v12_pdu_event<S>(
    id: &str,
    sender: &UserId,
    ev_type: TimelineEventType,
    state_key: Option<&str>,
    content: Box<RawJsonValue>,
    auth_events: &[S],
    prev_events: &[S],
) -> Arc<PduEvent>
where
    S: AsRef<str>,
{
    fn event_id(id: &str) -> EventId {
        if id.contains('$') { id.try_into().unwrap() } else { format!("${id}").try_into().unwrap() }
    }

    let ts = SERVER_TIMESTAMP.fetch_add(1, SeqCst);
    let auth_events = auth_events.iter().map(AsRef::as_ref).map(event_id).collect::<Vec<_>>();
    let prev_events = prev_events.iter().map(AsRef::as_ref).map(event_id).collect::<Vec<_>>();

    let state_key = state_key.map(ToOwned::to_owned);
    Arc::new(PduEvent {
        event_id: event_id(id),
        room_id: Some(v12_room_id().to_owned()),
        sender: sender.to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(ts.try_into().unwrap()),
        state_key,
        kind: ev_type,
        content,
        redacts: None,
        unsigned: BTreeMap::new(),
        auth_events,
        prev_events,
        depth: uint!(0),
        hashes: EventHash { sha256: "".to_owned() },
        signatures: ServerSignatures::default(),
        rejected: false,
    })
}

pub(crate) fn room_redaction_pdu_event<S>(
    id: &str,
    sender: &UserId,
    redacts: EventId,
    content: Box<RawJsonValue>,
    auth_events: &[S],
    prev_events: &[S],
) -> Arc<PduEvent>
where
    S: AsRef<str>,
{
    let ts = SERVER_TIMESTAMP.fetch_add(1, SeqCst);
    let id = if id.contains('$') { id.to_owned() } else { format!("${id}:foo") };
    let auth_events = auth_events.iter().map(AsRef::as_ref).map(event_id).collect::<Vec<_>>();
    let prev_events = prev_events.iter().map(AsRef::as_ref).map(event_id).collect::<Vec<_>>();

    Arc::new(PduEvent {
        event_id: id.try_into().unwrap(),
        room_id: Some(room_id().to_owned()),
        sender: sender.to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(ts.try_into().unwrap()),
        state_key: None,
        kind: TimelineEventType::RoomRedaction,
        content,
        redacts: Some(redacts),
        unsigned: BTreeMap::new(),
        auth_events,
        prev_events,
        depth: uint!(0),
        hashes: EventHash { sha256: "".to_owned() },
        signatures: ServerSignatures::default(),
        rejected: false,
    })
}

pub(crate) fn room_create_v12_pdu_event(
    id: &str,
    sender: &UserId,
    content: Box<RawJsonValue>,
) -> Arc<PduEvent> {
    let ts = SERVER_TIMESTAMP.fetch_add(1, SeqCst);
    let id = if id.contains('$') { id.to_owned() } else { format!("${id}") };

    Arc::new(PduEvent {
        event_id: id.try_into().unwrap(),
        room_id: None,
        sender: sender.to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(ts.try_into().unwrap()),
        state_key: Some(String::new()),
        kind: TimelineEventType::RoomCreate,
        content,
        redacts: None,
        unsigned: BTreeMap::new(),
        auth_events: vec![],
        prev_events: vec![],
        depth: uint!(0),
        hashes: EventHash { sha256: "".to_owned() },
        signatures: ServerSignatures::default(),
        rejected: false,
    })
}

/// Batch of initial events to use for incoming events in the v1-v11 room versions.
#[allow(non_snake_case)]
pub(crate) fn INITIAL_EVENTS() -> HashMap<EventId, Arc<PduEvent>> {
    vec![
        to_pdu_event::<&EventId>(
            "CREATE",
            alice(),
            TimelineEventType::RoomCreate,
            Some(""),
            to_raw_json_value(&json!({ "creator": alice() })).unwrap(),
            &[],
            &[],
        ),
        to_pdu_event(
            "IMA",
            alice(),
            TimelineEventType::RoomMember,
            Some(alice().as_str()),
            member_content_join(),
            &["CREATE"],
            &["CREATE"],
        ),
        to_pdu_event(
            "IPOWER",
            alice(),
            TimelineEventType::RoomPowerLevels,
            Some(""),
            to_raw_json_value(&json!({ "users": { alice(): 100 } })).unwrap(),
            &["CREATE", "IMA"],
            &["IMA"],
        ),
        to_pdu_event(
            "IJR",
            alice(),
            TimelineEventType::RoomJoinRules,
            Some(""),
            to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Public)).unwrap(),
            &["CREATE", "IMA", "IPOWER"],
            &["IPOWER"],
        ),
        to_pdu_event(
            "IMB",
            bob(),
            TimelineEventType::RoomMember,
            Some(bob().as_str()),
            member_content_join(),
            &["CREATE", "IJR", "IPOWER"],
            &["IJR"],
        ),
        to_pdu_event(
            "IMC",
            charlie(),
            TimelineEventType::RoomMember,
            Some(charlie().as_str()),
            member_content_join(),
            &["CREATE", "IJR", "IPOWER"],
            &["IMB"],
        ),
        to_pdu_event::<&EventId>(
            "START",
            charlie(),
            TimelineEventType::RoomMessage,
            Some("dummy"),
            to_raw_json_value(&json!({})).unwrap(),
            &[],
            &[],
        ),
        to_pdu_event::<&EventId>(
            "END",
            charlie(),
            TimelineEventType::RoomMessage,
            Some("dummy"),
            to_raw_json_value(&json!({})).unwrap(),
            &[],
            &[],
        ),
    ]
    .into_iter()
    .map(|ev| (ev.event_id().to_owned(), ev))
    .collect()
}

/// Batch of initial events to use for incoming events from room version 12 onwards.
#[allow(non_snake_case)]
pub(crate) fn INITIAL_V12_EVENTS() -> HashMap<EventId, Arc<PduEvent>> {
    vec![
        room_create_v12_pdu_event(
            "CREATE",
            alice(),
            to_raw_json_value(&json!({ "room_version": "12" })).unwrap(),
        ),
        to_v12_pdu_event(
            "IMA",
            alice(),
            TimelineEventType::RoomMember,
            Some(alice().as_str()),
            member_content_join(),
            &["CREATE"],
            &["CREATE"],
        ),
        to_v12_pdu_event(
            "IPOWER",
            alice(),
            TimelineEventType::RoomPowerLevels,
            Some(""),
            to_raw_json_value(&json!({})).unwrap(),
            &["CREATE", "IMA"],
            &["IMA"],
        ),
        to_v12_pdu_event(
            "IJR",
            alice(),
            TimelineEventType::RoomJoinRules,
            Some(""),
            to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Public)).unwrap(),
            &["CREATE", "IMA", "IPOWER"],
            &["IPOWER"],
        ),
        to_v12_pdu_event(
            "IMB",
            bob(),
            TimelineEventType::RoomMember,
            Some(bob().as_str()),
            member_content_join(),
            &["CREATE", "IJR", "IPOWER"],
            &["IJR"],
        ),
        to_v12_pdu_event(
            "IMC",
            charlie(),
            TimelineEventType::RoomMember,
            Some(charlie().as_str()),
            member_content_join(),
            &["CREATE", "IJR", "IPOWER"],
            &["IMB"],
        ),
        to_v12_pdu_event::<&EventId>(
            "START",
            charlie(),
            TimelineEventType::RoomMessage,
            Some("dummy"),
            to_raw_json_value(&json!({})).unwrap(),
            &[],
            &[],
        ),
        to_v12_pdu_event::<&EventId>(
            "END",
            charlie(),
            TimelineEventType::RoomMessage,
            Some("dummy"),
            to_raw_json_value(&json!({})).unwrap(),
            &[],
            &[],
        ),
    ]
    .into_iter()
    .map(|ev| (ev.event_id().to_owned(), ev))
    .collect()
}

// all graphs start with these input events
#[allow(non_snake_case)]
pub(crate) fn INITIAL_EVENTS_CREATE_ROOM() -> HashMap<EventId, Arc<PduEvent>> {
    vec![to_pdu_event::<&EventId>(
        "CREATE",
        alice(),
        TimelineEventType::RoomCreate,
        Some(""),
        to_raw_json_value(&json!({ "creator": alice() })).unwrap(),
        &[],
        &[],
    )]
    .into_iter()
    .map(|ev| (ev.event_id().to_owned(), ev))
    .collect()
}

#[allow(non_snake_case)]
pub(crate) fn INITIAL_EDGES() -> Vec<EventId> {
    vec!["START", "IMC", "IMB", "IJR", "IPOWER", "IMA", "CREATE"]
        .into_iter()
        .map(event_id)
        .collect::<Vec<_>>()
}

pub(crate) mod event {
    use std::collections::BTreeMap;

    use js_int::UInt;
    use ruma_common::{EventId, MilliSecondsSinceUnixEpoch, RoomId, ServerSignatures, UserId};
    use ruma_events::TimelineEventType;
    use serde::{Deserialize, Serialize};
    use serde_json::value::RawValue as RawJsonValue;

    use crate::Event;

    impl Event for PduEvent {
        type Id = EventId;

        fn event_id(&self) -> &Self::Id {
            &self.event_id
        }

        fn room_id(&self) -> Option<&RoomId> {
            self.room_id.as_ref()
        }

        fn sender(&self) -> &UserId {
            &self.sender
        }

        fn event_type(&self) -> &TimelineEventType {
            &self.kind
        }

        fn content(&self) -> &RawJsonValue {
            &self.content
        }

        fn origin_server_ts(&self) -> MilliSecondsSinceUnixEpoch {
            self.origin_server_ts
        }

        fn state_key(&self) -> Option<&str> {
            self.state_key.as_deref()
        }

        fn prev_events(&self) -> Box<dyn DoubleEndedIterator<Item = &Self::Id> + '_> {
            Box::new(self.prev_events.iter())
        }

        fn auth_events(&self) -> Box<dyn DoubleEndedIterator<Item = &Self::Id> + '_> {
            Box::new(self.auth_events.iter())
        }

        fn redacts(&self) -> Option<&Self::Id> {
            self.redacts.as_ref()
        }

        fn rejected(&self) -> bool {
            self.rejected
        }
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[allow(clippy::exhaustive_structs)]
    pub(crate) struct PduEvent {
        /// The ID of the event.
        pub(crate) event_id: EventId,

        /// The room this event belongs to.
        pub(crate) room_id: Option<RoomId>,

        /// The user id of the user who sent this event.
        pub(crate) sender: UserId,

        /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver
        /// of when this event was created.
        pub(crate) origin_server_ts: MilliSecondsSinceUnixEpoch,

        /// The event's type.
        #[serde(rename = "type")]
        pub(crate) kind: TimelineEventType,

        /// The event's content.
        pub(crate) content: Box<RawJsonValue>,

        /// A key that determines which piece of room state the event represents.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) state_key: Option<String>,

        /// Event IDs for the most recent events in the room that the homeserver was
        /// aware of when it created this event.
        pub(crate) prev_events: Vec<EventId>,

        /// The maximum depth of the `prev_events`, plus one.
        pub(crate) depth: UInt,

        /// Event IDs for the authorization events that would allow this event to be
        /// in the room.
        pub(crate) auth_events: Vec<EventId>,

        /// For redaction events, the ID of the event being redacted.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) redacts: Option<EventId>,

        /// Additional data added by the origin server but not covered by the
        /// signatures.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub(crate) unsigned: BTreeMap<String, Box<RawJsonValue>>,

        /// Content hashes of the PDU.
        pub(crate) hashes: EventHash,

        /// Signatures for the PDU.
        pub(crate) signatures: ServerSignatures,

        /// Whether the PDU was rejected.
        pub(crate) rejected: bool,
    }

    /// Content hashes of a PDU.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[allow(clippy::exhaustive_structs)]
    pub(crate) struct EventHash {
        /// The SHA-256 hash.
        pub(crate) sha256: String,
    }
}

/// Wrapper around a state map.
pub(crate) struct TestStateMap(HashMap<StateEventType, HashMap<String, Arc<PduEvent>>>);

impl TestStateMap {
    /// Construct a `TestStateMap` from the given event map.
    pub(crate) fn new(events: &HashMap<EventId, Arc<PduEvent>>) -> Self {
        let mut state_map: HashMap<StateEventType, HashMap<String, Arc<PduEvent>>> = HashMap::new();

        for event in events.values() {
            let event_type = StateEventType::from(event.event_type().to_string());

            state_map
                .entry(event_type)
                .or_default()
                .insert(event.state_key().unwrap().to_owned(), event.clone());
        }

        TestStateMap(state_map)
    }

    /// Get the event with the given event type and state key.
    pub(crate) fn get(
        &self,
        event_type: &StateEventType,
        state_key: &str,
    ) -> Option<&Arc<PduEvent>> {
        self.0.get(event_type)?.get(state_key)
    }

    /// A function to get a state event from this map.
    pub(crate) fn fetch_state_fn<'a>(
        &'a self,
    ) -> impl Fn(&StateEventType, &str) -> Option<&'a Arc<PduEvent>> + Copy {
        |event_type: &StateEventType, state_key: &str| self.get(event_type, state_key)
    }

    /// The `m.room.create` event contained in this map.
    ///
    /// Panics if there is no `m.room.create` event in this map.
    pub(crate) fn room_create_event(&self) -> RoomCreateEvent<&Arc<PduEvent>> {
        RoomCreateEvent::new(self.get(&StateEventType::RoomCreate, "").unwrap())
    }
}

/// Create an `m.room.third_party_invite` event with the given sender.
pub(crate) fn room_third_party_invite(sender: &UserId) -> Arc<PduEvent> {
    let content = json!({
        "display_name": "o...@g...",
        "key_validity_url": "https://identity.local/_matrix/identity/v2/pubkey/isvalid",
        "public_key": "Gb9ECWmEzf6FQbrBZ9w7lshQhqowtrbLDFw4rXAxZuE",
        "public_keys": [
            {
                "key_validity_url": "https://identity.local/_matrix/identity/v2/pubkey/isvalid",
                "public_key": "Gb9ECWmEzf6FQbrBZ9w7lshQhqowtrbLDFw4rXAxZuE"
            },
            {
                "key_validity_url": "https://identity.local/_matrix/identity/v2/pubkey/ephemeral/isvalid",
                "public_key": "Kxdvv7lo0O6JVI7yimFgmYPfpLGnctcpYjuypP5zx/c"
            }
        ]
    });

    to_pdu_event(
        "THIRDPARTY",
        sender,
        TimelineEventType::RoomThirdPartyInvite,
        Some("somerandomtoken"),
        to_raw_json_value(&content).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    )
}
