// Because of criterion `cargo bench` works,
// but if you use `cargo bench -- --save-baseline <name>`
// or pass any other args to it, it fails with the error
// `cargo bench unknown option --save-baseline`.
// To pass args to criterion, use this form
// `cargo bench --bench <name of the bench> -- --save-baseline <name>`.

#![allow(clippy::exhaustive_structs)]

use criterion::{Criterion, criterion_group, criterion_main};
use js_int::{int, uint};
use ruma_common::{
    MilliSecondsSinceUnixEpoch, RoomVersionId, owned_event_id,
    room_version_rules::{AuthorizationRules, StateResolutionV2Rules},
};
use ruma_events::StateEventType;
use ruma_state_res::{
    self as state_res,
    test_utils::{
        PublicChatInitialPdu, RoomMemberPduContent, RoomPowerLevelsPduContent, RoomTimelineFactory,
        UserFactory,
    },
    utils::{event_id_map::EventIdMap, event_id_set::EventIdSet},
};

fn reverse_topological_power_sort(c: &mut Criterion) {
    c.bench_function("reverse_topological_power_sort", |b| {
        let graph = EventIdMap::from([
            (owned_event_id!("$l"), EventIdSet::from([owned_event_id!("$o")])),
            (
                owned_event_id!("$m"),
                EventIdSet::from([owned_event_id!("$n"), owned_event_id!("$o")]),
            ),
            (owned_event_id!("$n"), EventIdSet::from([owned_event_id!("$o")])),
            (owned_event_id!("$o"), EventIdSet::new()), /* "o" has zero outgoing edges but 4
                                                         * incoming edges */
            (owned_event_id!("$p"), EventIdSet::from([owned_event_id!("$o")])),
        ]);
        b.iter(|| {
            let _ = state_res::reverse_topological_power_sort(&graph, |_id| {
                Ok((int!(0).into(), MilliSecondsSinceUnixEpoch(uint!(0))))
            });
        });
    });
}

fn resolution_shallow_auth_chain(c: &mut Criterion) {
    c.bench_function("resolve state of 6 events one fork", |b| {
        let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

        // On Charlie's fork, Charlie joins the room.
        let charlie_id = UserFactory::Charlie.user_id();
        factory.add_room_member(
            owned_event_id!("$room-member-charlie-join"),
            charlie_id.clone(),
            RoomMemberPduContent::Join,
        );
        let charlie_state_map = factory.state().clone();
        let charlie_auth_chain = factory.full_auth_chain(&charlie_state_map);

        // On Zara's fork, Zara joins the room.
        let zara_room_member_event_id = owned_event_id!("$room-member-zara-join");
        let zara_room_member_pdu = factory.add_room_member(
            zara_room_member_event_id.clone(),
            UserFactory::Zara.user_id(),
            RoomMemberPduContent::Join,
        );
        // Remove Charlie's event from the prev events and state map to create the fork.
        zara_room_member_pdu.prev_events.clear();
        zara_room_member_pdu.prev_events.insert(PublicChatInitialPdu::RoomMemberBobJoin.event_id());
        let mut zara_state_map = factory.state().clone();
        zara_state_map.remove(&(StateEventType::RoomMember, charlie_id.into()));
        let zara_auth_chain = factory.full_auth_chain(&zara_state_map);

        b.iter(|| {
            match state_res::resolve(
                &AuthorizationRules::V6,
                &StateResolutionV2Rules::V2_0,
                [&charlie_state_map, &zara_state_map],
                vec![charlie_auth_chain.clone(), zara_auth_chain.clone()],
                factory.get_fn(),
                |_| unreachable!(),
            ) {
                Ok(_) => {}
                Err(e) => panic!("{e}"),
            }
        });
    });
}

fn resolve_deeper_event_set(c: &mut Criterion) {
    c.bench_function("resolve state of 10 events 3 conflicting", |b| {
        let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

        let alice_id = UserFactory::Alice.user_id();
        let zara_id = UserFactory::Zara.user_id();

        // On Zara's fork, Zara joins the room.
        let zara_room_member_join_event_id = owned_event_id!("$room-member-zara-join");
        factory.add_room_member(
            zara_room_member_join_event_id.clone(),
            zara_id.clone(),
            RoomMemberPduContent::Join,
        );

        // On Alice's fork, Alice bans Zara.
        let zara_room_member_ban_pdu = factory.add_room_member(
            owned_event_id!("$room-member-zara-ban"),
            zara_id.clone(),
            RoomMemberPduContent::Ban { sender: alice_id.clone() },
        );
        // Remove the join event from the prev events and auth events for this fork.
        zara_room_member_ban_pdu.prev_events.clear();
        zara_room_member_ban_pdu
            .prev_events
            .insert(PublicChatInitialPdu::RoomMemberBobJoin.event_id());
        zara_room_member_ban_pdu.auth_events.remove(&zara_room_member_join_event_id);

        // Finally Alice changes the power levels to promote Bob, which both forks agree on.
        let room_power_levels_event_id = owned_event_id!("$room-power-levels-bob");
        let room_power_levels_pdu = factory.add_room_power_levels(
            room_power_levels_event_id.clone(),
            alice_id,
            RoomPowerLevelsPduContent::User { user_id: UserFactory::Bob.user_id(), value: 50 },
        );
        // Because there is a fork, it has 2 prev_events.
        room_power_levels_pdu.prev_events.insert(zara_room_member_join_event_id.clone());

        // Zara's state map should have the join event instead of the ban event.
        let mut zara_state_map = factory.state().clone();
        zara_state_map
            .insert((StateEventType::RoomMember, zara_id.into()), zara_room_member_join_event_id);
        let zara_auth_chain = factory.full_auth_chain(&zara_state_map);

        // The factory's state map matches Alice's.
        let alice_state_map = factory.state().clone();
        let alice_auth_chain = factory.full_auth_chain(&alice_state_map);

        b.iter(|| {
            state_res::resolve(
                &AuthorizationRules::V6,
                &StateResolutionV2Rules::V2_0,
                [&zara_state_map, &alice_state_map],
                vec![zara_auth_chain.clone(), alice_auth_chain.clone()],
                factory.get_fn(),
                |_| unreachable!(),
            )
            .unwrap_or_else(|_| panic!("resolution failed during benchmarking"));
        });
    });
}

criterion_group!(
    benches,
    reverse_topological_power_sort,
    resolution_shallow_auth_chain,
    resolve_deeper_event_set
);

criterion_main!(benches);
