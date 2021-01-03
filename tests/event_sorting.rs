use std::collections::BTreeMap;

use ruma::{
    events::EventType,
    identifiers::{EventId, RoomVersionId},
};
use state_res::{is_power_event, StateMap};

mod utils;
use utils::{room_id, TestStore, INITIAL_EVENTS};

fn shuffle(list: &mut [EventId]) {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    for i in 1..list.len() {
        let j = rng.gen_range(0, list.len());
        list.swap(i, j);
    }
}

fn test_event_sort() {
    let mut events = INITIAL_EVENTS();
    let store = TestStore(events.clone());

    let event_map = events
        .values()
        .map(|ev| ((ev.kind(), ev.state_key()), ev.clone()))
        .collect::<StateMap<_>>();

    let auth_chain = &[] as &[_];

    let power_events = event_map
        .values()
        .filter(|pdu| is_power_event(&pdu))
        .map(|pdu| pdu.event_id().clone())
        .collect::<Vec<_>>();

    // This is a TODO in conduit
    // TODO these events are not guaranteed to be sorted but they are resolved, do
    // we need the auth_chain
    let sorted_power_events = state_res::StateResolution::reverse_topological_power_sort(
        &room_id(),
        &power_events,
        &mut events,
        &store,
        &auth_chain,
    );

    // This is a TODO in conduit
    // TODO we may be able to skip this since they are resolved according to spec
    let resolved_power = state_res::StateResolution::iterative_auth_check(
        &room_id(),
        &RoomVersionId::Version6,
        &sorted_power_events,
        &BTreeMap::new(), // unconflicted events
        &mut events,
        &store,
    )
    .expect("iterative auth check failed on resolved events");

    // don't remove any events so we know it sorts them all correctly
    let mut events_to_sort = events.keys().cloned().collect::<Vec<_>>();

    shuffle(&mut events_to_sort);

    let power_level = resolved_power.get(&(EventType::RoomPowerLevels, Some("".to_string())));

    let sorted_event_ids = state_res::StateResolution::mainline_sort(
        &room_id(),
        &events_to_sort,
        power_level,
        &mut events,
        &store,
    );

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
        sorted_event_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
    )
}

#[test]
fn test_sort() {
    for _ in 0..20 {
        // since we shuffle the eventIds before we sort them introducing randomness
        // seems like we should test this a few times
        test_event_sort()
    }
}
