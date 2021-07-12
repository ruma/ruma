use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use rand::seq::SliceRandom;
use ruma_events::EventType;
use ruma_state_res::{is_power_event, room_version::RoomVersion, StateMap, StateResolution};

mod utils;
use utils::INITIAL_EVENTS;

fn test_event_sort() {
    let events = INITIAL_EVENTS();

    let event_map = events
        .values()
        .map(|ev| ((ev.kind(), ev.state_key()), ev.clone()))
        .collect::<StateMap<_>>();

    let auth_chain = BTreeSet::new();

    let power_events = event_map
        .values()
        .filter(|pdu| is_power_event(pdu))
        .map(|pdu| pdu.event_id().clone())
        .collect::<Vec<_>>();

    // This is a TODO in conduit
    // TODO these events are not guaranteed to be sorted but they are resolved, do
    // we need the auth_chain
    let sorted_power_events =
        StateResolution::reverse_topological_power_sort(&power_events, &auth_chain, |id| {
            events.get(id).map(Arc::clone)
        });

    // This is a TODO in conduit
    // TODO we may be able to skip this since they are resolved according to spec
    let resolved_power = StateResolution::iterative_auth_check(
        &RoomVersion::version_6(),
        &sorted_power_events,
        &BTreeMap::new(), // unconflicted events
        |id| events.get(id).map(Arc::clone),
    )
    .expect("iterative auth check failed on resolved events");

    // don't remove any events so we know it sorts them all correctly
    let mut events_to_sort = events.keys().cloned().collect::<Vec<_>>();

    events_to_sort.shuffle(&mut rand::thread_rng());

    let power_level = resolved_power.get(&(EventType::RoomPowerLevels, "".to_owned()));

    let sorted_event_ids = StateResolution::mainline_sort(&events_to_sort, power_level, |id| {
        events.get(id).map(Arc::clone)
    });

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
