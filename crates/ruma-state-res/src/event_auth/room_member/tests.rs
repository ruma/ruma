use ruma_common::{serde::Base64, Signatures};
use ruma_events::{
    room::{
        join_rules::{JoinRule, Restricted, RoomJoinRulesEventContent},
        member::{MembershipState, RoomMemberEventContent, SignedContent, ThirdPartyInvite},
        third_party_invite::RoomThirdPartyInviteEventContent,
    },
    StateEventType, TimelineEventType,
};
use serde_json::{json, value::to_raw_value as to_raw_json_value};

use super::check_room_member;
use crate::{
    test_utils::{
        alice, bob, charlie, ella, event_id, event_map_to_state_map, init_subscriber,
        member_content_ban, member_content_join, to_pdu_event, zara, INITIAL_EVENTS,
        INITIAL_EVENTS_CREATE_ROOM,
    },
    RoomVersion,
};

#[test]
fn missing_state_key() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        None,
        member_content_join(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Event should have a state key.
    assert!(!check_room_member(incoming_event, &RoomVersion::V6, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn missing_membership() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        to_raw_json_value(&json!({})).unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Content should at least include `membership`.
    assert!(!check_room_member(incoming_event, &RoomVersion::V6, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn join_after_create_creator_match() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        alice(),
        TimelineEventType::RoomMember,
        Some(alice().as_str()),
        member_content_join(),
        &["CREATE"],
        &["CREATE"],
    );

    let init_events = INITIAL_EVENTS_CREATE_ROOM();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Before v11, the `creator` of `m.room.create` must be the same as the state key.
    assert!(check_room_member(incoming_event, &RoomVersion::V6, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn join_after_create_creator_mismatch() {
    let _guard = init_subscriber();

    let requester = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        member_content_join(),
        &["CREATE"],
        &["CREATE"],
    );

    let init_events = INITIAL_EVENTS_CREATE_ROOM();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Before v11, the `creator` of `m.room.create` must be the same as the state key.
    assert!(
        !check_room_member(requester, &RoomVersion::V6, room_create_event, fetch_state).unwrap()
    );
}

#[test]
fn join_after_create_sender_match() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        alice(),
        TimelineEventType::RoomMember,
        Some(alice().as_str()),
        member_content_join(),
        &["CREATE"],
        &["CREATE"],
    );

    let init_events = INITIAL_EVENTS_CREATE_ROOM();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Since v11, the `sender` of `m.room.create` must be the same as the state key.
    assert!(check_room_member(incoming_event, &RoomVersion::V11, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn join_after_create_sender_mismatch() {
    let _guard = init_subscriber();

    let requester = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        member_content_join(),
        &["CREATE"],
        &["CREATE"],
    );

    let init_events = INITIAL_EVENTS_CREATE_ROOM();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Since v11, the `sender` of `m.room.create` must be the same as the state key.
    assert!(
        !check_room_member(requester, &RoomVersion::V11, room_create_event, fetch_state).unwrap()
    );
}

#[test]
fn join_sender_state_key_mismatch() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(alice().as_str()),
        member_content_join(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // For join events, the sender must be the same as the state key.
    assert!(!check_room_member(incoming_event, &RoomVersion::V6, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn join_banned() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        member_content_join(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IMC")).unwrap() = to_pdu_event(
        "IMC",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        member_content_ban(),
        &["CREATE", "IJR", "IPOWER"],
        &["IMB"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // A user cannot join if they are banned.
    assert!(!check_room_member(incoming_event, &RoomVersion::V6, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn join_invite_join_rule_already_joined() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        member_content_join(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IJR")).unwrap() = to_pdu_event(
        "IJR",
        alice(),
        TimelineEventType::RoomJoinRules,
        Some(""),
        to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Invite)).unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // A user can send a join event in a room with `invite` join rule if they already joined.
    assert!(check_room_member(incoming_event, &RoomVersion::V6, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn join_knock_join_rule_already_invited() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        member_content_join(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IMC")).unwrap() = to_pdu_event(
        "IMC",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Invite)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IMB"],
    );
    *init_events.get_mut(&event_id("IJR")).unwrap() = to_pdu_event(
        "IJR",
        alice(),
        TimelineEventType::RoomJoinRules,
        Some(""),
        to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Knock)).unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Since v7, a user can send a join event in a room with `knock` join rule if they are were
    // invited.
    assert!(check_room_member(incoming_event, &RoomVersion::V7, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn join_knock_join_rule_not_supported() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        member_content_join(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IJR")).unwrap() = to_pdu_event(
        "IJR",
        alice(),
        TimelineEventType::RoomJoinRules,
        Some(""),
        to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Knock)).unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Before v7, a user CANNOT send a join event in a room with `knock` join rule. Servers should
    // not allow that join rule if it's not supported by the room version, but this is good
    // for coverage.
    assert!(!check_room_member(incoming_event, &RoomVersion::V6, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn join_restricted_join_rule_not_supported() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        member_content_join(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IJR")).unwrap() = to_pdu_event(
        "IJR",
        alice(),
        TimelineEventType::RoomJoinRules,
        Some(""),
        to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Restricted(Restricted::new(
            vec![],
        ))))
        .unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Before v8, a user CANNOT send a join event in a room with `restricted` join rule. Servers
    // should not allow that join rule if it's not supported by the room version, but this is good
    // for coverage.
    assert!(!check_room_member(incoming_event, &RoomVersion::V6, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn join_knock_restricted_join_rule_not_supported() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        member_content_join(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IJR")).unwrap() = to_pdu_event(
        "IJR",
        alice(),
        TimelineEventType::RoomJoinRules,
        Some(""),
        to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::KnockRestricted(
            Restricted::new(vec![]),
        )))
        .unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Before v10, a user CANNOT send a join event in a room with `knock_restricted` join rule.
    // Servers should not allow that join rule if it's not supported by the room version, but
    // this is good for coverage.
    assert!(!check_room_member(incoming_event, &RoomVersion::V6, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn join_restricted_join_rule_already_joined() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        member_content_join(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IJR")).unwrap() = to_pdu_event(
        "IJR",
        alice(),
        TimelineEventType::RoomJoinRules,
        Some(""),
        to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Restricted(Restricted::new(
            vec![],
        ))))
        .unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Since v8, a user can send a join event in a room with `restricted` join rule if they already
    // joined.
    assert!(check_room_member(incoming_event, &RoomVersion::V8, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn join_knock_restricted_join_rule_already_invited() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        member_content_join(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IMC")).unwrap() = to_pdu_event(
        "IMC",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Invite)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IMB"],
    );
    *init_events.get_mut(&event_id("IJR")).unwrap() = to_pdu_event(
        "IJR",
        alice(),
        TimelineEventType::RoomJoinRules,
        Some(""),
        to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::KnockRestricted(
            Restricted::new(vec![]),
        )))
        .unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Since v10, a user can send a join event in a room with `knock_restricted` join rule if they
    // were invited.
    assert!(check_room_member(incoming_event, &RoomVersion::V10, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn join_restricted_join_rule_missing_join_authorised_via_users_server() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        ella(),
        TimelineEventType::RoomMember,
        Some(ella().as_str()),
        member_content_join(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IJR")).unwrap() = to_pdu_event(
        "IJR",
        alice(),
        TimelineEventType::RoomJoinRules,
        Some(""),
        to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Restricted(Restricted::new(
            vec![],
        ))))
        .unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Since v8, a user CANNOT join event in a room with `restricted` join rule if there is no
    // `join_authorised_via_users_server` property.
    assert!(!check_room_member(incoming_event, &RoomVersion::V8, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn join_restricted_join_rule_authorised_via_user_not_in_room() {
    let _guard = init_subscriber();

    let mut content = RoomMemberEventContent::new(MembershipState::Join);
    content.join_authorized_via_users_server = Some(zara().to_owned());

    let incoming_event = to_pdu_event(
        "HELLO",
        ella(),
        TimelineEventType::RoomMember,
        Some(ella().as_str()),
        to_raw_json_value(&content).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IJR")).unwrap() = to_pdu_event(
        "IJR",
        alice(),
        TimelineEventType::RoomJoinRules,
        Some(""),
        to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Restricted(Restricted::new(
            vec![],
        ))))
        .unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Since v8, a user CANNOT join event in a room with `restricted` join rule if they were
    // authorized by a user not in the room.
    assert!(!check_room_member(incoming_event, &RoomVersion::V8, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn join_restricted_join_rule_authorised_via_user_with_not_enough_power() {
    let _guard = init_subscriber();

    let mut content = RoomMemberEventContent::new(MembershipState::Join);
    content.join_authorized_via_users_server = Some(charlie().to_owned());

    let incoming_event = to_pdu_event(
        "HELLO",
        ella(),
        TimelineEventType::RoomMember,
        Some(ella().as_str()),
        to_raw_json_value(&content).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IPOWER")).unwrap() = to_pdu_event(
        "IPOWER",
        alice(),
        TimelineEventType::RoomPowerLevels,
        Some(""),
        to_raw_json_value(&json!({ "users": { alice(): 100 }, "invite": 50 })).unwrap(),
        &["CREATE", "IMA"],
        &["IMA"],
    );
    *init_events.get_mut(&event_id("IJR")).unwrap() = to_pdu_event(
        "IJR",
        alice(),
        TimelineEventType::RoomJoinRules,
        Some(""),
        to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Restricted(Restricted::new(
            vec![],
        ))))
        .unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Since v8, a user CANNOT join event in a room with `restricted` join rule if they were
    // authorized by a user with not enough power.
    assert!(!check_room_member(incoming_event, &RoomVersion::V8, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn join_restricted_join_rule_authorised_via_user() {
    let _guard = init_subscriber();

    let mut content = RoomMemberEventContent::new(MembershipState::Join);
    content.join_authorized_via_users_server = Some(charlie().to_owned());

    let incoming_event = to_pdu_event(
        "HELLO",
        ella(),
        TimelineEventType::RoomMember,
        Some(ella().as_str()),
        to_raw_json_value(&content).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IJR")).unwrap() = to_pdu_event(
        "IJR",
        alice(),
        TimelineEventType::RoomJoinRules,
        Some(""),
        to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Restricted(Restricted::new(
            vec![],
        ))))
        .unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Since v8, a user CANNOT join event in a room with `restricted` join rule if they were
    // authorized by a user with not enough power.
    assert!(check_room_member(incoming_event, &RoomVersion::V9, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn join_public_join_rule() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        ella(),
        TimelineEventType::RoomMember,
        Some(ella().as_str()),
        member_content_join(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // A user can join a room with a `public` join rule.
    assert!(check_room_member(incoming_event, &RoomVersion::V9, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn invite_via_third_party_invite_banned() {
    let _guard = init_subscriber();

    let mut content = RoomMemberEventContent::new(MembershipState::Invite);
    content.third_party_invite = Some(ThirdPartyInvite::new(
        "e..@p..".to_owned(),
        SignedContent::new(Signatures::new(), ella().to_owned(), "somerandomtoken".to_owned()),
    ));

    let incoming_event = to_pdu_event(
        "HELLO",
        ella(),
        TimelineEventType::RoomMember,
        Some(ella().as_str()),
        to_raw_json_value(&content).unwrap(),
        &["CREATE", "BAN", "IPOWER"],
        &["BAN"],
    );

    let mut init_events = INITIAL_EVENTS();
    init_events.insert(
        event_id("BAN"),
        to_pdu_event(
            "BAN",
            alice(),
            TimelineEventType::RoomMember,
            Some(ella().as_str()),
            member_content_ban(),
            &["CREATE", "IJR", "IPOWER"],
            &["IPOWER"],
        ),
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // A user cannot be invited via third party invite if they were banned.
    assert!(!check_room_member(incoming_event, &RoomVersion::V9, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn invite_via_third_party_invite_missing_signed() {
    let _guard = init_subscriber();

    let content = json!({
        "membership": "invite",
        "third_party_invite": {
            "display_name": "e..@p..",
        },
    });

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(ella().as_str()),
        to_raw_json_value(&content).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Third party invite content must have a `joined` property.
    check_room_member(incoming_event, &RoomVersion::V9, room_create_event, fetch_state)
        .unwrap_err();
}

#[test]
fn invite_via_third_party_invite_missing_mxid() {
    let _guard = init_subscriber();

    let content = json!({
        "membership": "invite",
        "third_party_invite": {
            "display_name": "e..@p..",
            "signed": {
                "token": "somerandomtoken",
            },
        },
    });

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(ella().as_str()),
        to_raw_json_value(&content).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Third party invite content must have a `joined.mxid` property.
    check_room_member(incoming_event, &RoomVersion::V9, room_create_event, fetch_state)
        .unwrap_err();
}

#[test]
fn invite_via_third_party_invite_missing_token() {
    let _guard = init_subscriber();

    let content = json!({
        "membership": "invite",
        "third_party_invite": {
            "display_name": "e..@p..",
            "signed": {
                "mxid": ella(),
            },
        },
    });

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(ella().as_str()),
        to_raw_json_value(&content).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Third party invite content must have a `joined.token` property.
    check_room_member(incoming_event, &RoomVersion::V9, room_create_event, fetch_state)
        .unwrap_err();
}

#[test]
fn invite_via_third_party_invite_mxid_mismatch() {
    let _guard = init_subscriber();

    let mut content = RoomMemberEventContent::new(MembershipState::Invite);
    content.third_party_invite = Some(ThirdPartyInvite::new(
        "z..@p..".to_owned(),
        SignedContent::new(Signatures::new(), zara().to_owned(), "somerandomtoken".to_owned()),
    ));

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(ella().as_str()),
        to_raw_json_value(&content).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // `mxid` of third party invite must match state key.
    assert!(!check_room_member(incoming_event, &RoomVersion::V9, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn invite_via_third_party_invite_missing_room_third_party_invite() {
    let _guard = init_subscriber();

    let mut content = RoomMemberEventContent::new(MembershipState::Invite);
    content.third_party_invite = Some(ThirdPartyInvite::new(
        "e..@p..".to_owned(),
        SignedContent::new(Signatures::new(), ella().to_owned(), "somerandomtoken".to_owned()),
    ));

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(ella().as_str()),
        to_raw_json_value(&content).unwrap(),
        &["CREATE", "IJR", "IPOWER", "THIRDPARTY"],
        &["THIRDPARTY"],
    );

    let mut init_events = INITIAL_EVENTS();
    init_events.insert(
        event_id("THIRD_PARTY"),
        to_pdu_event(
            "THIRDPARTY",
            charlie(),
            TimelineEventType::RoomThirdPartyInvite,
            Some("wrong_token"),
            to_raw_json_value(&RoomThirdPartyInviteEventContent::new(
                "e..@p..".to_owned(),
                "http://host.local/check/public_key".to_owned(),
                Base64::new(b"public_key".to_vec()),
            ))
            .unwrap(),
            &["CREATE", "IJR", "IPOWER"],
            &["IPOWER"],
        ),
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // There must be an `m.room.third_party_invite` event with the same token in the state.
    assert!(!check_room_member(incoming_event, &RoomVersion::V9, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn invite_via_third_party_invite_room_third_party_invite_sender_mismatch() {
    let _guard = init_subscriber();

    let mut content = RoomMemberEventContent::new(MembershipState::Invite);
    content.third_party_invite = Some(ThirdPartyInvite::new(
        "e..@p..".to_owned(),
        SignedContent::new(Signatures::new(), ella().to_owned(), "somerandomtoken".to_owned()),
    ));

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(ella().as_str()),
        to_raw_json_value(&content).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    init_events.insert(
        event_id("THIRD_PARTY"),
        to_pdu_event(
            "THIRDPARTY",
            bob(),
            TimelineEventType::RoomThirdPartyInvite,
            Some("somerandomtoken"),
            to_raw_json_value(&RoomThirdPartyInviteEventContent::new(
                "e..@p..".to_owned(),
                "http://host.local/check/public_key".to_owned(),
                Base64::new(b"public_key".to_vec()),
            ))
            .unwrap(),
            &["CREATE", "IJR", "IPOWER"],
            &["IPOWER"],
        ),
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // `mxid` of third party invite must match state key.
    assert!(!check_room_member(incoming_event, &RoomVersion::V9, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn invite_via_third_party_invite_with_room_third_party_invite() {
    // FIXME: for now the code doesn't check the signatures, this test will fail once it does.
    let _guard = init_subscriber();

    let mut content = RoomMemberEventContent::new(MembershipState::Invite);
    content.third_party_invite = Some(ThirdPartyInvite::new(
        "e..@p..".to_owned(),
        SignedContent::new(Signatures::new(), ella().to_owned(), "somerandomtoken".to_owned()),
    ));

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(ella().as_str()),
        to_raw_json_value(&content).unwrap(),
        &["CREATE", "IJR", "IPOWER", "THIRDPARTY"],
        &["THIRDPARTY"],
    );

    let mut init_events = INITIAL_EVENTS();
    init_events.insert(
        event_id("THIRD_PARTY"),
        to_pdu_event(
            "THIRDPARTY",
            charlie(),
            TimelineEventType::RoomThirdPartyInvite,
            Some("somerandomtoken"),
            to_raw_json_value(&RoomThirdPartyInviteEventContent::new(
                "e..@p..".to_owned(),
                "http://host.local/check/public_key".to_owned(),
                Base64::new(b"public_key".to_vec()),
            ))
            .unwrap(),
            &["CREATE", "IJR", "IPOWER"],
            &["IPOWER"],
        ),
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Valid third party invite works.
    assert!(check_room_member(incoming_event, &RoomVersion::V9, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn invite_sender_not_joined() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        zara(),
        TimelineEventType::RoomMember,
        Some(ella().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Invite)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // The sender of the invite must have joined the room.
    assert!(!check_room_member(incoming_event, &RoomVersion::V9, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn invite_banned() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        alice(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Invite)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IMC")).unwrap() = to_pdu_event(
        "IMC",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        member_content_ban(),
        &["CREATE", "IJR", "IPOWER"],
        &["IMB"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // The sender of the invite must have joined the room.
    assert!(!check_room_member(incoming_event, &RoomVersion::V9, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn invite_already_joined() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        alice(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Invite)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // The sender of the invite must have joined the room.
    assert!(!check_room_member(incoming_event, &RoomVersion::V9, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn invite_sender_not_enough_power() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(ella().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Invite)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IPOWER")).unwrap() = to_pdu_event(
        "IPOWER",
        alice(),
        TimelineEventType::RoomPowerLevels,
        Some(""),
        to_raw_json_value(&json!({ "users": { alice(): 100 }, "invite": 50 })).unwrap(),
        &["CREATE", "IMA"],
        &["IMA"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // The sender must have enough power to invite in the room.
    assert!(!check_room_member(incoming_event, &RoomVersion::V9, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn invite() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(ella().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Invite)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // The invite is valid.
    assert!(check_room_member(incoming_event, &RoomVersion::V9, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn leave_after_leave() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        ella(),
        TimelineEventType::RoomMember,
        Some(ella().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Leave)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // User can only leave after `invite`, `join` or `knock`.
    assert!(!check_room_member(incoming_event, &RoomVersion::V9, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn leave_after_join() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Leave)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // User can leave after join.
    assert!(check_room_member(incoming_event, &RoomVersion::V9, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn leave_after_invite() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Leave)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IMC")).unwrap() = to_pdu_event(
        "IMC",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Invite)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IMB"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // User can leave after invite.
    assert!(check_room_member(incoming_event, &RoomVersion::V9, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn leave_after_knock() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Leave)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IMC")).unwrap() = to_pdu_event(
        "IMC",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Knock)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IMB"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // User can leave after knock.
    assert!(check_room_member(incoming_event, &RoomVersion::V9, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn leave_after_knock_not_supported() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Leave)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IMC")).unwrap() = to_pdu_event(
        "IMC",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Knock)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IMB"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // User can't leave if the room version does not support knocking. Servers should not allow that
    // membership if it's not supported by the room version, but this is good for coverage.
    assert!(!check_room_member(incoming_event, &RoomVersion::V6, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn leave_kick_sender_left() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        zara(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Leave)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // User can't kick if not joined.
    assert!(!check_room_member(incoming_event, &RoomVersion::V6, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn leave_unban_not_enough_power() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        bob(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Leave)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IMC")).unwrap() = to_pdu_event(
        "IMC",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        member_content_ban(),
        &["CREATE", "IJR", "IPOWER"],
        &["IMB"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // User can't unban if not enough power.
    assert!(!check_room_member(incoming_event, &RoomVersion::V6, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn leave_unban() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        alice(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Leave)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IMC")).unwrap() = to_pdu_event(
        "IMC",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        member_content_ban(),
        &["CREATE", "IJR", "IPOWER"],
        &["IMB"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // User can unban with enough power.
    assert!(check_room_member(incoming_event, &RoomVersion::V6, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn leave_kick_not_enough_power() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        bob(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Leave)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // User can't kick if not enough power for it.
    assert!(!check_room_member(incoming_event, &RoomVersion::V6, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn leave_kick_greater_power() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        bob(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Leave)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IPOWER")).unwrap() = to_pdu_event(
        "IPOWER",
        alice(),
        TimelineEventType::RoomPowerLevels,
        Some(""),
        to_raw_json_value(&json!({
            "users": {
                alice(): 100,
                bob(): 50,
                charlie(): 60,
            },
        }))
        .unwrap(),
        &["CREATE", "IMA"],
        &["IMA"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Can't kick user with greater power level.
    assert!(!check_room_member(incoming_event, &RoomVersion::V6, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn leave_kick_same_power() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        bob(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Leave)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IPOWER")).unwrap() = to_pdu_event(
        "IPOWER",
        alice(),
        TimelineEventType::RoomPowerLevels,
        Some(""),
        to_raw_json_value(&json!({
            "users": {
                alice(): 100,
                bob(): 50,
                charlie(): 50,
            },
        }))
        .unwrap(),
        &["CREATE", "IMA"],
        &["IMA"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Can't kick user with same power level.
    assert!(!check_room_member(incoming_event, &RoomVersion::V6, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn leave_kick() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        alice(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Leave)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Can kick user with enough power.
    assert!(check_room_member(incoming_event, &RoomVersion::V6, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn ban_sender_not_joined() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        ella(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        member_content_ban(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Can't ban user if not in room.
    assert!(!check_room_member(incoming_event, &RoomVersion::V6, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn ban_not_enough_power() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(bob().as_str()),
        member_content_ban(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Can't ban user if not enough power.
    assert!(!check_room_member(incoming_event, &RoomVersion::V6, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn ban_greater_power() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(bob().as_str()),
        member_content_ban(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IPOWER")).unwrap() = to_pdu_event(
        "IPOWER",
        alice(),
        TimelineEventType::RoomPowerLevels,
        Some(""),
        to_raw_json_value(&json!({
            "users": {
                alice(): 100,
                bob(): 60,
                charlie(): 50,
            },
        }))
        .unwrap(),
        &["CREATE", "IMA"],
        &["IMA"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Can't ban user with greater power level.
    assert!(!check_room_member(incoming_event, &RoomVersion::V6, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn ban_same_power() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(bob().as_str()),
        member_content_ban(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IPOWER")).unwrap() = to_pdu_event(
        "IPOWER",
        alice(),
        TimelineEventType::RoomPowerLevels,
        Some(""),
        to_raw_json_value(&json!({
            "users": {
                alice(): 100,
                bob(): 50,
                charlie(): 50,
            },
        }))
        .unwrap(),
        &["CREATE", "IMA"],
        &["IMA"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Can't ban user with same power level.
    assert!(!check_room_member(incoming_event, &RoomVersion::V6, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn ban() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        alice(),
        TimelineEventType::RoomMember,
        Some(bob().as_str()),
        member_content_ban(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // Can ban user with enough power.
    assert!(check_room_member(incoming_event, &RoomVersion::V6, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn knock_public_join_rule() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        ella(),
        TimelineEventType::RoomMember,
        Some(ella().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Knock)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // User can't knock if join rule is not `knock` or `knock_restricted`.
    assert!(!check_room_member(incoming_event, &RoomVersion::V11, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn knock_knock_join_rule() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        ella(),
        TimelineEventType::RoomMember,
        Some(ella().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Knock)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IJR")).unwrap() = to_pdu_event(
        "IJR",
        alice(),
        TimelineEventType::RoomJoinRules,
        Some(""),
        to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Knock)).unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // User can knock if room version supports it.
    assert!(check_room_member(incoming_event, &RoomVersion::V7, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn knock_knock_join_rule_not_supported() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        ella(),
        TimelineEventType::RoomMember,
        Some(ella().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Knock)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IJR")).unwrap() = to_pdu_event(
        "IJR",
        alice(),
        TimelineEventType::RoomJoinRules,
        Some(""),
        to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Knock)).unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // User CANNOT knock if room version doesn't support it.
    assert!(!check_room_member(incoming_event, &RoomVersion::V5, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn knock_knock_restricted_join_rule() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        ella(),
        TimelineEventType::RoomMember,
        Some(ella().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Knock)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IJR")).unwrap() = to_pdu_event(
        "IJR",
        alice(),
        TimelineEventType::RoomJoinRules,
        Some(""),
        to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::KnockRestricted(
            Restricted::new(vec![]),
        )))
        .unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // User can knock if room version supports it.
    assert!(check_room_member(incoming_event, &RoomVersion::V10, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn knock_knock_restricted_join_rule_not_supported() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        ella(),
        TimelineEventType::RoomMember,
        Some(ella().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Knock)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IJR")).unwrap() = to_pdu_event(
        "IJR",
        alice(),
        TimelineEventType::RoomJoinRules,
        Some(""),
        to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::KnockRestricted(
            Restricted::new(vec![]),
        )))
        .unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // User CANNOT knock if room version doesn't support it.
    assert!(!check_room_member(incoming_event, &RoomVersion::V5, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn knock_sender_state_key_mismatch() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        zara(),
        TimelineEventType::RoomMember,
        Some(ella().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Knock)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IJR")).unwrap() = to_pdu_event(
        "IJR",
        alice(),
        TimelineEventType::RoomJoinRules,
        Some(""),
        to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Knock)).unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // User cannot knock if state key doesn't match sender.
    assert!(!check_room_member(incoming_event, &RoomVersion::V7, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn knock_after_ban() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Knock)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IJR")).unwrap() = to_pdu_event(
        "IJR",
        alice(),
        TimelineEventType::RoomJoinRules,
        Some(""),
        to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Knock)).unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );
    *init_events.get_mut(&event_id("IMC")).unwrap() = to_pdu_event(
        "IMC",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        member_content_ban(),
        &["CREATE", "IJR", "IPOWER"],
        &["IMB"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // User cannot knock if banned.
    assert!(!check_room_member(incoming_event, &RoomVersion::V7, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn knock_after_invite() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Knock)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IJR")).unwrap() = to_pdu_event(
        "IJR",
        alice(),
        TimelineEventType::RoomJoinRules,
        Some(""),
        to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Knock)).unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );
    *init_events.get_mut(&event_id("IMC")).unwrap() = to_pdu_event(
        "IMC",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Invite)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IMB"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // User cannot knock after being invited.
    assert!(!check_room_member(incoming_event, &RoomVersion::V7, room_create_event, fetch_state)
        .unwrap());
}

#[test]
fn knock_after_join() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMember,
        Some(charlie().as_str()),
        to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Knock)).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IJR")).unwrap() = to_pdu_event(
        "IJR",
        alice(),
        TimelineEventType::RoomJoinRules,
        Some(""),
        to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Knock)).unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let auth_events = event_map_to_state_map(&init_events);
    let room_create_event = auth_events.get(&StateEventType::RoomCreate).unwrap().get("").unwrap();
    let fetch_state =
        |ty: &StateEventType, key: &str| auth_events.get(ty).and_then(|map| map.get(key)).cloned();

    // User cannot knock after being invited.
    assert!(!check_room_member(incoming_event, &RoomVersion::V7, room_create_event, fetch_state)
        .unwrap());
}
