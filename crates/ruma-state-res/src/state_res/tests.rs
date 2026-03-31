use std::collections::{HashMap, HashSet};

use js_int::{int, uint};
use maplit::{hashmap, hashset};
use rand::seq::SliceRandom;
use ruma_common::{
    MilliSecondsSinceUnixEpoch, OwnedEventId, room_version_rules::AuthorizationRules,
};
use ruma_events::StateEventType;
use test_log::test;

use super::{EventTypeExt, StateMap, is_power_event};
use crate::{
    Event,
    test_utils::{INITIAL_EVENTS, event_id},
};

fn test_event_sort() {
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
fn test_reverse_topological_power_sort() {
    let graph = hashmap! {
        event_id("l") => hashset![event_id("o")],
        event_id("m") => hashset![event_id("n"), event_id("o")],
        event_id("n") => hashset![event_id("o")],
        event_id("o") => hashset![], // "o" has zero outgoing edges but 4 incoming edges
        event_id("p") => hashset![event_id("o")],
    };

    let res = crate::reverse_topological_power_sort(&graph, |_id| {
        Ok((int!(0).into(), MilliSecondsSinceUnixEpoch(uint!(0))))
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
