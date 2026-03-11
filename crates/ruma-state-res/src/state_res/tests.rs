use js_int::{int, uint};
use maplit::hashmap;
use ruma_common::{
    MilliSecondsSinceUnixEpoch, RoomVersionId, owned_event_id,
    room_version_rules::AuthorizationRules,
};
use ruma_events::StateEventType;
use test_log::test;

use super::{StateMap, is_power_event};
use crate::{test_utils::RoomTimelineFactory, utils::event_id_set::EventIdSet};

#[test]
fn test_sort_power_events() {
    // Because we use the keys and values of a `HashMap` to get the events to sort, their order
    // before sorting changes every time, so let's run this several times.
    for _ in 0..20 {
        let factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

        let power_events = factory
            .pdus()
            .values()
            .filter(|pdu| is_power_event(pdu))
            .map(|pdu| pdu.event_id.clone())
            .collect();
        let full_conflicted_set = factory.pdus().keys().cloned().collect();

        let sorted_events = super::sort_power_events(
            power_events,
            &full_conflicted_set,
            &AuthorizationRules::V6,
            factory.get_fn(),
        )
        .unwrap();

        assert_eq!(
            sorted_events,
            ["$room-create", "$room-member-alice-join", "$room-power-levels", "$room-join-rules"]
        );
    }
}

#[test]
fn test_mainline_sort() {
    // Because we use the keys and values of a `HashMap` to get the events to sort, their order
    // before sorting changes every time, so let's run this several times.
    for _ in 0..20 {
        let factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V10);

        let events = factory.pdus().keys().cloned().collect::<Vec<_>>();
        let power_level =
            factory.state_event_id(&StateEventType::RoomPowerLevels, "").unwrap().clone();

        let sorted_events =
            super::mainline_sort(&events, Some(power_level), factory.get_fn()).unwrap();

        assert_eq!(
            sorted_events,
            [
                "$room-create",
                "$room-member-alice-join",
                "$room-power-levels",
                "$room-join-rules",
                "$room-member-bob-join"
            ]
        );
    }
}

#[test]
fn test_reverse_topological_power_sort() {
    let graph = hashmap! {
        owned_event_id!("$l") => EventIdSet::from([owned_event_id!("$o")]),
        owned_event_id!("$m") => EventIdSet::from([owned_event_id!("$n"), owned_event_id!("$o")]),
        owned_event_id!("$n") => EventIdSet::from([owned_event_id!("$o")]),
        owned_event_id!("$o") => EventIdSet::new(), // "o" has zero outgoing edges but 4 incoming edges
        owned_event_id!("$p") => EventIdSet::from([owned_event_id!("$o")]),
    };

    let sorted = crate::reverse_topological_power_sort(&graph, |_id| {
        Ok((int!(0).into(), MilliSecondsSinceUnixEpoch(uint!(0))))
    })
    .unwrap();

    assert_eq!(sorted, ["$o", "$l", "$n", "$m", "$p"],);
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
