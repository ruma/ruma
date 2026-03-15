use ruma_common::{
    RoomVersionId, owned_event_id, owned_room_id,
    room::{AllowRule, RoomMembership},
    room_version_rules::AuthorizationRules,
};
use ruma_events::{
    TimelineEventType,
    room::join_rules::{JoinRule, Restricted},
};
use serde_json::json;
use test_log::test;

use super::check_room_member;
use crate::{
    events::RoomMemberEvent,
    test_utils::{
        Pdu, RoomCreatePduBuilder, RoomMemberPduContent, RoomPowerLevelsPduContent,
        RoomTimelineFactory, UserFactory,
    },
};

#[test]
fn missing_state_key() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    let mut pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-join"),
        UserFactory::Charlie.user_id(),
        RoomMemberPduContent::Join,
    );
    pdu.state_key.take();

    // Event should have a state key.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V6,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "missing `state_key` field in `m.room.member` event"
    );
}

#[test]
fn missing_membership() {
    let factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    let charlie_id = UserFactory::Charlie.user_id();
    let pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-member-charlie-join"),
        charlie_id.clone(),
        TimelineEventType::RoomMember,
        charlie_id.into(),
        json!({}),
    );

    // Content should at least include `membership`.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V6,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "missing or invalid `membership` field in `m.room.member` event: missing field `membership` at line 1 column 2"
    );
}

#[test]
fn join_after_create_creator_match() {
    let mut factory = RoomCreatePduBuilder::new(RoomVersionId::V6).build_factory();

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-alice-join"),
        UserFactory::Alice.user_id(),
        RoomMemberPduContent::Join,
    );

    // Before v11, the `creator` of `m.room.create` must be the same as the state key.
    check_room_member(
        RoomMemberEvent::new(pdu),
        &AuthorizationRules::V6,
        factory.room_create_pdu(),
        factory.state_event_fn(),
    )
    .unwrap();
}

#[test]
fn join_after_create_creator_mismatch() {
    let mut factory = RoomCreatePduBuilder::new(RoomVersionId::V6).build_factory();

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-bob-join"),
        UserFactory::Bob.user_id(),
        RoomMemberPduContent::Join,
    );

    // Before v11, the `creator` of `m.room.create` must be the same as the state key.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V6,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "no `m.room.join_rules` event in current state"
    );
}

#[test]
fn join_after_create_sender_match() {
    let mut factory = RoomCreatePduBuilder::new(RoomVersionId::V11).build_factory();

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-alice-join"),
        UserFactory::Alice.user_id(),
        RoomMemberPduContent::Join,
    );

    // Since v11, the `sender` of `m.room.create` must be the same as the state key.
    check_room_member(
        RoomMemberEvent::new(pdu),
        &AuthorizationRules::V11,
        factory.room_create_pdu(),
        factory.state_event_fn(),
    )
    .unwrap();
}

#[test]
fn join_after_create_sender_mismatch() {
    let mut factory = RoomCreatePduBuilder::new(RoomVersionId::V11).build_factory();

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-bob-join"),
        UserFactory::Bob.user_id(),
        RoomMemberPduContent::Join,
    );

    // Since v11, the `sender` of `m.room.create` must be the same as the state key.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V11,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "no `m.room.join_rules` event in current state"
    );
}

#[test]
fn join_sender_state_key_mismatch() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    let mut pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-join"),
        UserFactory::Charlie.user_id(),
        RoomMemberPduContent::Join,
    );
    pdu.sender = UserFactory::Alice.user_id();

    // For join events, the sender must be the same as the state key.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V6,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "sender of join event must match target user"
    );
}

#[test]
fn join_banned() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);
    let charlie_id = UserFactory::Charlie.user_id();

    factory.add_room_member(
        owned_event_id!("$room-member-charlie-ban"),
        charlie_id.clone(),
        RoomMemberPduContent::Ban { sender: UserFactory::Alice.user_id() },
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-join"),
        charlie_id,
        RoomMemberPduContent::Join,
    );

    // A user cannot join if they are banned.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V6,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "banned user cannot join room"
    );
}

#[test]
fn join_invite_join_rule_already_joined() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    factory.add_room_join_rules(
        owned_event_id!("$room-join-rules-invite"),
        UserFactory::Alice.user_id(),
        JoinRule::Invite,
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-bob-displayname"),
        UserFactory::Bob.user_id(),
        RoomMemberPduContent::DisplayName { displayname: "Bob".to_owned() },
    );

    // A user can send a join event in a room with `invite` join rule if they already joined.
    check_room_member(
        RoomMemberEvent::new(pdu),
        &AuthorizationRules::V6,
        factory.room_create_pdu(),
        factory.state_event_fn(),
    )
    .unwrap();
}

#[test]
fn join_knock_join_rule_already_invited() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V7);
    let alice_id = UserFactory::Alice.user_id();
    let charlie_id = UserFactory::Charlie.user_id();

    factory.add_room_join_rules(
        owned_event_id!("$room-join-rules-invite"),
        alice_id.clone(),
        JoinRule::Invite,
    );
    factory.add_room_member(
        owned_event_id!("$room-member-charlie-invite"),
        charlie_id.clone(),
        RoomMemberPduContent::Invite { sender: alice_id },
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-join"),
        charlie_id,
        RoomMemberPduContent::Join,
    );

    // Since v7, a user can send a join event in a room with `knock` join rule if they are were
    // invited.
    check_room_member(
        RoomMemberEvent::new(pdu),
        &AuthorizationRules::V7,
        factory.room_create_pdu(),
        factory.state_event_fn(),
    )
    .unwrap();
}

#[test]
fn join_knock_join_rule_not_supported() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    factory.add_room_join_rules(
        owned_event_id!("$room-join-rules-knock"),
        UserFactory::Alice.user_id(),
        JoinRule::Knock,
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-join"),
        UserFactory::Charlie.user_id(),
        RoomMemberPduContent::Join,
    );

    // Before v7, a user CANNOT send a join event in a room with `knock` join rule. Servers should
    // not allow that join rule if it's not supported by the room version, but this is good
    // for coverage.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V6,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "cannot join a room that is not `public`"
    );
}

#[test]
fn join_restricted_join_rule_not_supported() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    factory.add_room_join_rules(
        owned_event_id!("$room-join-rules-restricted"),
        UserFactory::Alice.user_id(),
        JoinRule::Restricted(Restricted::new(vec![AllowRule::RoomMembership(
            RoomMembership::new(owned_room_id!("!space:matrix.local")),
        )])),
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-join"),
        UserFactory::Charlie.user_id(),
        RoomMemberPduContent::Join,
    );

    // Before v8, a user CANNOT send a join event in a room with `restricted` join rule. Servers
    // should not allow that join rule if it's not supported by the room version, but this is good
    // for coverage.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V6,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "cannot join a room that is not `public`"
    );
}

#[test]
fn join_knock_restricted_join_rule_not_supported() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    factory.add_room_join_rules(
        owned_event_id!("$room-join-rules-knock-restricted"),
        UserFactory::Alice.user_id(),
        JoinRule::KnockRestricted(Restricted::new(vec![AllowRule::RoomMembership(
            RoomMembership::new(owned_room_id!("!space:matrix.local")),
        )])),
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-join"),
        UserFactory::Charlie.user_id(),
        RoomMemberPduContent::Join,
    );

    // Before v10, a user CANNOT send a join event in a room with `knock_restricted` join rule.
    // Servers should not allow that join rule if it's not supported by the room version, but
    // this is good for coverage.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V6,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "cannot join a room that is not `public`"
    );
}

#[test]
fn join_restricted_join_rule_already_joined() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);

    factory.add_room_join_rules(
        owned_event_id!("$room-join-rules-restricted"),
        UserFactory::Alice.user_id(),
        JoinRule::Restricted(Restricted::new(vec![AllowRule::RoomMembership(
            RoomMembership::new(owned_room_id!("!space:matrix.local")),
        )])),
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-bob-displayname"),
        UserFactory::Bob.user_id(),
        RoomMemberPduContent::DisplayName { displayname: "Bob".to_owned() },
    );

    // Since v8, a user can send a join event in a room with `restricted` join rule if they already
    // joined.
    check_room_member(
        RoomMemberEvent::new(pdu),
        &AuthorizationRules::V8,
        factory.room_create_pdu(),
        factory.state_event_fn(),
    )
    .unwrap();
}

#[test]
fn join_knock_restricted_join_rule_already_invited() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V10);
    let alice_id = UserFactory::Alice.user_id();
    let charlie_id = UserFactory::Charlie.user_id();

    factory.add_room_join_rules(
        owned_event_id!("$room-join-rules-knock-restricted"),
        alice_id.clone(),
        JoinRule::KnockRestricted(Restricted::new(vec![AllowRule::RoomMembership(
            RoomMembership::new(owned_room_id!("!space:matrix.local")),
        )])),
    );
    factory.add_room_member(
        owned_event_id!("$room-member-charlie-invite"),
        charlie_id.clone(),
        RoomMemberPduContent::Invite { sender: alice_id },
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-join"),
        UserFactory::Charlie.user_id(),
        RoomMemberPduContent::Join,
    );

    // Since v10, a user can send a join event in a room with `knock_restricted` join rule if they
    // were invited.
    check_room_member(
        RoomMemberEvent::new(pdu),
        &AuthorizationRules::V10,
        factory.room_create_pdu(),
        factory.state_event_fn(),
    )
    .unwrap();
}

#[test]
fn join_restricted_join_rule_missing_join_authorised_via_users_server() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);

    factory.add_room_join_rules(
        owned_event_id!("$room-join-rules-restricted"),
        UserFactory::Alice.user_id(),
        JoinRule::Restricted(Restricted::new(vec![AllowRule::RoomMembership(
            RoomMembership::new(owned_room_id!("!space:matrix.local")),
        )])),
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-join"),
        UserFactory::Charlie.user_id(),
        RoomMemberPduContent::Join,
    );

    // Since v8, a user CANNOT join event in a room with `restricted` join rule if there is no
    // `join_authorised_via_users_server` property.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V8,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "cannot join restricted room without `join_authorised_via_users_server` field if not invited"
    );
}

#[test]
fn join_restricted_join_rule_authorised_via_user_not_in_room() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);

    factory.add_room_join_rules(
        owned_event_id!("$room-join-rules-restricted"),
        UserFactory::Alice.user_id(),
        JoinRule::Restricted(Restricted::new(vec![AllowRule::RoomMembership(
            RoomMembership::new(owned_room_id!("!space:matrix.local")),
        )])),
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-join"),
        UserFactory::Charlie.user_id(),
        RoomMemberPduContent::JoinAuthorized { via_users_server: UserFactory::Zara.user_id() },
    );

    // Since v8, a user CANNOT join event in a room with `restricted` join rule if they were
    // authorized by a user not in the room.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V8,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "`join_authorised_via_users_server` is not joined"
    );
}

#[test]
fn join_restricted_join_rule_authorised_via_user_with_not_enough_power() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);
    let alice_id = UserFactory::Alice.user_id();

    factory.add_room_join_rules(
        owned_event_id!("$room-join-rules-restricted"),
        alice_id.clone(),
        JoinRule::Restricted(Restricted::new(vec![AllowRule::RoomMembership(
            RoomMembership::new(owned_room_id!("!space:matrix.local")),
        )])),
    );
    factory.add_room_power_levels(
        owned_event_id!("$room-power-levels-invite"),
        alice_id,
        RoomPowerLevelsPduContent::Invite { value: 50 },
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-join"),
        UserFactory::Charlie.user_id(),
        RoomMemberPduContent::JoinAuthorized { via_users_server: UserFactory::Bob.user_id() },
    );

    // Since v8, a user CANNOT join event in a room with `restricted` join rule if they were
    // authorized by a user with not enough power.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V8,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "`join_authorised_via_users_server` does not have enough power"
    );
}

#[test]
fn join_restricted_join_rule_authorised_via_user() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);
    let alice_id = UserFactory::Alice.user_id();

    // Check various contents that might not match the definition of `m.room.join_rules` in the
    // spec, to ensure that we only care about the `join_rule` field.
    let join_rules_to_check = [
        // Valid content, but we don't care about the allow rules.
        json!({
            "allow": [],
            "join_rule": "restricted",
        }),
        // Invalid room ID, real-life example from <https://github.com/ruma/ruma/issues/1867>.
        json!({
            "allow": [{
                "room_id": "",
                "type": "m.room_membership",
            }],
            "join_rule": "restricted",
        }),
        // Missing room ID.
        json!({
            "allow": [{
                "type": "m.room_membership",
            }],
            "join_rule": "restricted",
        }),
    ];

    let mut pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-join"),
        UserFactory::Charlie.user_id(),
        RoomMemberPduContent::JoinAuthorized { via_users_server: UserFactory::Bob.user_id() },
    );

    for (i, content) in join_rules_to_check.iter().enumerate() {
        let mut join_rules_pdu = Pdu::with_minimal_state_fields(
            format!("$room-join-rules-restricted-{i}").try_into().unwrap(),
            alice_id.clone(),
            TimelineEventType::RoomJoinRules,
            String::new(),
            content,
        );
        factory.prepare_to_add_pdu(&mut join_rules_pdu);
        factory.add_pdu(join_rules_pdu);

        factory.prepare_to_add_pdu(&mut pdu);

        // Since v8, a user can join event in a room with `restricted` join rule if they were
        // authorized by a user with enough power.
        check_room_member(
            RoomMemberEvent::new(&pdu),
            &AuthorizationRules::V8,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap();
    }
}

#[test]
fn join_public_join_rule() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);
    let charlie_id = UserFactory::Charlie.user_id();

    // Check various contents that might not match the definition of `m.room.member` in the
    // spec, to ensure that we only care about a few fields.
    let contents_to_check = [
        // Valid content.
        json!({
            "membership": "join",
        }),
        // Invalid displayname.
        json!({
            "membership": "join",
            "displayname": 203,
        }),
        // Invalid is_direct.
        json!({
            "membership": "join",
            "is_direct": "yes",
        }),
    ];

    for (i, content) in contents_to_check.iter().enumerate() {
        let mut pdu = Pdu::with_minimal_state_fields(
            format!("$room-member-charlie-join-{i}").try_into().unwrap(),
            charlie_id.clone(),
            TimelineEventType::RoomMember,
            charlie_id.to_string(),
            content,
        );
        factory.prepare_to_add_pdu(&mut pdu);

        // A user can join a room with a `public` join rule.
        check_room_member(
            RoomMemberEvent::new(&pdu),
            &AuthorizationRules::V8,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap();
    }
}

#[test]
fn invite_via_third_party_invite_banned() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);

    factory.add_room_member(
        owned_event_id!("$room-member-zara-ban"),
        UserFactory::Zara.user_id(),
        RoomMemberPduContent::Ban { sender: UserFactory::Alice.user_id() },
    );

    let pdu = factory.create_room_member_third_party_invite();

    // A user cannot be invited via third party invite if they were banned.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V8,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "cannot invite user that is banned"
    );
}

#[test]
fn invite_via_third_party_invite_missing_signed() {
    let factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);

    let content = json!({
        "membership": "invite",
        "third_party_invite": {
            "display_name": "z..@o..",
        },
    });
    let pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-member-zara-invite"),
        UserFactory::Bob.user_id(),
        TimelineEventType::RoomMember,
        UserFactory::Zara.user_id().into(),
        content,
    );

    // Third party invite content must have a `joined` property.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V8,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "invalid `third_party_invite` field in `m.room.member` event: missing field `signed` at line 1 column 70"
    );
}

#[test]
fn invite_via_third_party_invite_missing_mxid() {
    let factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);

    let content = json!({
        "membership": "invite",
        "third_party_invite": {
            "display_name": "z..@o..",
            "signed": {
                "token": "uniquetoken",
            },
        },
    });
    let pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-member-zara-invite"),
        UserFactory::Bob.user_id(),
        TimelineEventType::RoomMember,
        UserFactory::Zara.user_id().into(),
        content,
    );

    // Third party invite content must have a `joined.mxid` property.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V8,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "missing `mxid` field in `third_party_invite.signed` of `m.room.member` event"
    );
}

#[test]
fn invite_via_third_party_invite_missing_token() {
    let factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);
    let zara_id = UserFactory::Zara.user_id();

    let content = json!({
        "membership": "invite",
        "third_party_invite": {
            "display_name": "e..@p..",
            "signed": {
                "mxid": zara_id,
            },
        },
    });
    let pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-member-zara-invite"),
        UserFactory::Bob.user_id(),
        TimelineEventType::RoomMember,
        zara_id.into(),
        content,
    );

    // Third party invite content must have a `joined.token` property.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V8,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "missing `token` field in `third_party_invite.signed` of `m.room.member` event"
    );
}

#[test]
fn invite_via_third_party_invite_mxid_mismatch() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);

    let mut pdu = factory.create_room_member_third_party_invite();
    pdu.state_key = Some(UserFactory::Charlie.user_id().into());

    // `mxid` of third party invite must match state key.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V8,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "third-party invite mxid does not match target user"
    );
}

#[test]
fn invite_via_third_party_invite_missing_room_third_party_invite() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);

    let pdu = factory.create_room_member_third_party_invite();

    // There must be an `m.room.third_party_invite` event with the same token in the state.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V8,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "no `m.room.third_party_invite` in room state matches the token"
    );
}

#[test]
fn invite_via_third_party_invite_room_third_party_invite_sender_mismatch() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);

    let room_third_party_invite_pdu = factory.add_room_third_party_invite();
    room_third_party_invite_pdu.sender = UserFactory::Alice.user_id();

    let pdu = factory.create_room_member_third_party_invite();

    // `sender` of invite must match `sender` of `m.room.third_party_invite`.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V8,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "sender of `m.room.third_party_invite` does not match sender of `m.room.member`"
    );
}

#[test]
fn invite_via_third_party_invite_with_missing_signatures() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);
    let bob_id = UserFactory::Bob.user_id();
    let zara_id = UserFactory::Zara.user_id();

    factory.add_room_third_party_invite();

    let content = json!({
        "membership": "invite",
        "third_party_invite": {
            "display_name": "z...@o...",
            "signed": {
                "mxid": zara_id,
                "sender": bob_id,
                "token": "uniquetoken",
            }
        }
    });
    let mut pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-member-zara-invite"),
        bob_id,
        TimelineEventType::RoomMember,
        zara_id.into(),
        content,
    );
    factory.prepare_to_add_pdu(&mut pdu);

    // `signed` must have a `signatures` field.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V8,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "missing `signatures` field in `third_party_invite.signed` of `m.room.member` event"
    );
}

#[test]
fn invite_via_third_party_invite_with_room_empty_signatures() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);
    let bob_id = UserFactory::Bob.user_id();
    let zara_id = UserFactory::Zara.user_id();

    factory.add_room_third_party_invite();

    let content = json!({
        "membership": "invite",
        "third_party_invite": {
            "display_name": "z...@o...",
            "signed": {
                "mxid": zara_id,
                "sender": bob_id,
                "token": "uniquetoken",
                "signatures": {},
            }
        }
    });
    let mut pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-member-zara-invite"),
        bob_id,
        TimelineEventType::RoomMember,
        zara_id.into(),
        content,
    );
    factory.prepare_to_add_pdu(&mut pdu);

    // There is no signature to verify, we need at least one.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V8,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "no signature on third-party invite matches a public key in `m.room.third_party_invite` event"
    );
}

#[test]
fn invite_via_third_party_invite_with_wrong_signature() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);
    let bob_id = UserFactory::Bob.user_id();
    let zara_id = UserFactory::Zara.user_id();

    factory.add_room_third_party_invite();

    let content = json!({
        "membership": "invite",
        "third_party_invite": {
            "display_name": "z...@o...",
            "signed": {
                "mxid": zara_id,
                "sender": bob_id,
                "token": "uniquetoken",
                "signatures": {
                    "identity.local": {
                        "ed25519:0": "ClearlyWrongSignature",
                    }
                },
            }
        }
    });
    let mut pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-member-zara-invite"),
        bob_id,
        TimelineEventType::RoomMember,
        zara_id.into(),
        content,
    );
    factory.prepare_to_add_pdu(&mut pdu);

    // No public key will manage to verify the signature.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V8,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "no signature on third-party invite matches a public key in `m.room.third_party_invite` event"
    );
}

#[test]
fn invite_via_third_party_invite_with_wrong_signing_algorithm() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);
    let bob_id = UserFactory::Bob.user_id();
    let zara_id = UserFactory::Zara.user_id();

    factory.add_room_third_party_invite();

    let content = json!({
        "membership": "invite",
        "third_party_invite": {
            "display_name": "z...@o...",
            "signed": {
                "mxid": zara_id,
                "sender": bob_id,
                "token": "uniquetoken",
                "signatures": {
                    "identity.local": {
                        "unknown:0": "EyW7uaJagmhIQg7DUFpiJv9ur8h8DkDSjyV6f5MlROJrrkg8JElBFKr2iTQY9x+A6OauQdNy7L9T4xgzIZVbCA",
                    }
                },
            }
        }
    });
    let mut pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-member-zara-invite"),
        bob_id,
        TimelineEventType::RoomMember,
        zara_id.into(),
        content,
    );
    factory.prepare_to_add_pdu(&mut pdu);

    // Can't verify a signature with an unsupported algorithm, so there is no signature to verify.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V8,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "no signature on third-party invite matches a public key in `m.room.third_party_invite` event"
    );
}

#[test]
fn invite_via_third_party_invite() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);

    factory.add_room_third_party_invite();

    let pdu = factory.create_room_member_third_party_invite();

    // Valid third party invite works.
    check_room_member(
        RoomMemberEvent::new(pdu),
        &AuthorizationRules::V8,
        factory.room_create_pdu(),
        factory.state_event_fn(),
    )
    .unwrap();
}

#[test]
fn invite_sender_not_joined() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-invite"),
        UserFactory::Charlie.user_id(),
        RoomMemberPduContent::Invite { sender: UserFactory::Zara.user_id() },
    );

    // The sender of the invite must have joined the room.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V8,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "cannot invite user if sender is not joined"
    );
}

#[test]
fn invite_banned() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);
    let charlie_id = UserFactory::Charlie.user_id();

    factory.add_room_member(
        owned_event_id!("$room-member-charlie-ban"),
        charlie_id.clone(),
        RoomMemberPduContent::Ban { sender: UserFactory::Alice.user_id() },
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-invite"),
        charlie_id,
        RoomMemberPduContent::Invite { sender: UserFactory::Bob.user_id() },
    );

    // The target user must not be banned.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V8,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "cannot invite user that is joined or banned"
    );
}

#[test]
fn invite_already_joined() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-invite"),
        UserFactory::Bob.user_id(),
        RoomMemberPduContent::Invite { sender: UserFactory::Alice.user_id() },
    );

    // The target user must not have joined the room.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V8,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "cannot invite user that is joined or banned"
    );
}

#[test]
fn invite_sender_not_enough_power() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);

    factory.add_room_power_levels(
        owned_event_id!("$room-power-levels-invite"),
        UserFactory::Alice.user_id(),
        RoomPowerLevelsPduContent::Invite { value: 50 },
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-invite"),
        UserFactory::Charlie.user_id(),
        RoomMemberPduContent::Invite { sender: UserFactory::Bob.user_id() },
    );

    // The sender must have enough power to invite in the room.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V8,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "sender does not have enough power to invite"
    );
}

#[test]
fn invite() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-invite"),
        UserFactory::Charlie.user_id(),
        RoomMemberPduContent::Invite { sender: UserFactory::Bob.user_id() },
    );

    // The invite is valid.
    check_room_member(
        RoomMemberEvent::new(pdu),
        &AuthorizationRules::V8,
        factory.room_create_pdu(),
        factory.state_event_fn(),
    )
    .unwrap();
}

#[test]
fn leave_after_leave() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-leave"),
        UserFactory::Charlie.user_id(),
        RoomMemberPduContent::Leave,
    );

    // User can only leave after `invite`, `join` or `knock`.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V8,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "cannot leave if not joined, invited or knocked"
    );
}

#[test]
fn leave_after_join() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-bob-leave"),
        UserFactory::Bob.user_id(),
        RoomMemberPduContent::Leave,
    );

    // User can leave after join.
    check_room_member(
        RoomMemberEvent::new(pdu),
        &AuthorizationRules::V8,
        factory.room_create_pdu(),
        factory.state_event_fn(),
    )
    .unwrap();
}

#[test]
fn leave_after_invite() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);
    let charlie_id = UserFactory::Charlie.user_id();

    factory.add_room_member(
        owned_event_id!("$room-member-charlie-invite"),
        charlie_id.clone(),
        RoomMemberPduContent::Invite { sender: UserFactory::Bob.user_id() },
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-leave"),
        charlie_id,
        RoomMemberPduContent::Leave,
    );

    // User can leave after invite.
    check_room_member(
        RoomMemberEvent::new(pdu),
        &AuthorizationRules::V8,
        factory.room_create_pdu(),
        factory.state_event_fn(),
    )
    .unwrap();
}

#[test]
fn leave_after_knock() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V8);
    let charlie_id = UserFactory::Charlie.user_id();

    factory.add_room_member(
        owned_event_id!("$room-member-charlie-knock"),
        charlie_id.clone(),
        RoomMemberPduContent::Knock,
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-leave"),
        charlie_id,
        RoomMemberPduContent::Leave,
    );

    // User can leave after knock.
    check_room_member(
        RoomMemberEvent::new(pdu),
        &AuthorizationRules::V8,
        factory.room_create_pdu(),
        factory.state_event_fn(),
    )
    .unwrap();
}

#[test]
fn leave_after_knock_not_supported() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);
    let charlie_id = UserFactory::Charlie.user_id();

    factory.add_room_member(
        owned_event_id!("$room-member-charlie-knock"),
        charlie_id.clone(),
        RoomMemberPduContent::Knock,
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-leave"),
        charlie_id,
        RoomMemberPduContent::Leave,
    );

    // User can't leave if the room version does not support knocking. Servers should not allow that
    // membership if it's not supported by the room version, but this is good for coverage.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V6,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "cannot leave if not joined, invited or knocked"
    );
}

#[test]
fn leave_kick_sender_left() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-bob-leave"),
        UserFactory::Bob.user_id(),
        RoomMemberPduContent::Kick { sender: UserFactory::Charlie.user_id() },
    );

    // User can't kick if not joined.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V6,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "cannot kick if sender is not joined"
    );
}

#[test]
fn leave_unban_not_enough_power() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);
    let charlie_id = UserFactory::Charlie.user_id();

    factory.add_room_member(
        owned_event_id!("$room-member-charlie-join"),
        charlie_id.clone(),
        RoomMemberPduContent::Join,
    );
    factory.add_room_member(
        owned_event_id!("$room-member-charlie-ban"),
        charlie_id.clone(),
        RoomMemberPduContent::Ban { sender: UserFactory::Alice.user_id() },
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-kick"),
        charlie_id,
        RoomMemberPduContent::Kick { sender: UserFactory::Bob.user_id() },
    );

    // User can't unban if not enough power.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V6,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "sender does not have enough power to unban"
    );
}

#[test]
fn leave_unban() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);
    let bob_id = UserFactory::Bob.user_id();
    let alice_id = UserFactory::Alice.user_id();

    factory.add_room_member(
        owned_event_id!("$room-member-charlie-ban"),
        bob_id.clone(),
        RoomMemberPduContent::Ban { sender: alice_id.clone() },
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-kick"),
        bob_id,
        RoomMemberPduContent::Kick { sender: alice_id },
    );

    // User can unban with enough power.
    check_room_member(
        RoomMemberEvent::new(pdu),
        &AuthorizationRules::V6,
        factory.room_create_pdu(),
        factory.state_event_fn(),
    )
    .unwrap();
}

#[test]
fn leave_kick_not_enough_power() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-alice-kick"),
        UserFactory::Alice.user_id(),
        RoomMemberPduContent::Kick { sender: UserFactory::Bob.user_id() },
    );

    // User can't kick if not enough power for it.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V6,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "sender does not have enough power to kick target user"
    );
}

#[test]
fn leave_kick_greater_power() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);
    let alice_id = UserFactory::Alice.user_id();
    let bob_id = UserFactory::Bob.user_id();

    factory.add_room_power_levels(
        owned_event_id!("$room-power-levels-bob"),
        alice_id.clone(),
        RoomPowerLevelsPduContent::User { user_id: bob_id.clone(), value: 50 },
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-alice-kick"),
        alice_id,
        RoomMemberPduContent::Kick { sender: bob_id },
    );

    // Can't kick user with greater power level.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V6,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "sender does not have enough power to kick target user"
    );
}

#[test]
fn leave_kick_same_power() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);
    let alice_id = UserFactory::Alice.user_id();
    let bob_id = UserFactory::Bob.user_id();

    factory.add_room_power_levels(
        owned_event_id!("$room-power-levels-bob"),
        alice_id.clone(),
        RoomPowerLevelsPduContent::User { user_id: bob_id.clone(), value: 100 },
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-alice-kick"),
        alice_id,
        RoomMemberPduContent::Kick { sender: bob_id },
    );

    // Can't kick user with same power level.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V6,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "sender does not have enough power to kick target user"
    );
}

#[test]
fn leave_kick() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-alice-kick"),
        UserFactory::Bob.user_id(),
        RoomMemberPduContent::Kick { sender: UserFactory::Alice.user_id() },
    );

    // Can kick user with enough power.
    check_room_member(
        RoomMemberEvent::new(pdu),
        &AuthorizationRules::V6,
        factory.room_create_pdu(),
        factory.state_event_fn(),
    )
    .unwrap();
}

#[test]
fn ban_sender_not_joined() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-bob-ban"),
        UserFactory::Bob.user_id(),
        RoomMemberPduContent::Ban { sender: UserFactory::Zara.user_id() },
    );

    // Can't ban user if not in room.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V6,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "cannot ban if sender is not joined"
    );
}

#[test]
fn ban_not_enough_power() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-bob-ban"),
        UserFactory::Alice.user_id(),
        RoomMemberPduContent::Ban { sender: UserFactory::Bob.user_id() },
    );

    // Can't ban user if not enough power.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V6,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "sender does not have enough power to ban target user"
    );
}

#[test]
fn ban_greater_power() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);
    let alice_id = UserFactory::Alice.user_id();
    let bob_id = UserFactory::Bob.user_id();

    factory.add_room_power_levels(
        owned_event_id!("$room-power-levels-bob"),
        alice_id.clone(),
        RoomPowerLevelsPduContent::User { user_id: bob_id.clone(), value: 50 },
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-bob-ban"),
        alice_id,
        RoomMemberPduContent::Ban { sender: bob_id },
    );

    // Can't ban user with greater power level.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V6,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "sender does not have enough power to ban target user"
    );
}

#[test]
fn ban_same_power() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);
    let alice_id = UserFactory::Alice.user_id();
    let bob_id = UserFactory::Bob.user_id();

    factory.add_room_power_levels(
        owned_event_id!("$room-power-levels-bob"),
        alice_id.clone(),
        RoomPowerLevelsPduContent::User { user_id: bob_id.clone(), value: 100 },
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-bob-ban"),
        alice_id,
        RoomMemberPduContent::Ban { sender: bob_id },
    );

    // Can't ban user with same power level.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V6,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "sender does not have enough power to ban target user"
    );
}

#[test]
fn ban() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-bob-ban"),
        UserFactory::Bob.user_id(),
        RoomMemberPduContent::Ban { sender: UserFactory::Alice.user_id() },
    );

    // Can ban user with enough power.
    check_room_member(
        RoomMemberEvent::new(pdu),
        &AuthorizationRules::V6,
        factory.room_create_pdu(),
        factory.state_event_fn(),
    )
    .unwrap();
}

#[test]
fn knock_public_join_rule() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V11);

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-zara-knock"),
        UserFactory::Zara.user_id(),
        RoomMemberPduContent::Knock,
    );

    // User can't knock if join rule is not `knock` or `knock_restricted`.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V11,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "join rule is not set to knock or knock_restricted, knocking is not allowed"
    );
}

#[test]
fn knock_knock_join_rule() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V7);

    factory.add_room_join_rules(
        owned_event_id!("$room-join-rules-knock"),
        UserFactory::Alice.user_id(),
        JoinRule::Knock,
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-zara-knock"),
        UserFactory::Zara.user_id(),
        RoomMemberPduContent::Knock,
    );

    // User can knock if room version supports it.
    check_room_member(
        RoomMemberEvent::new(pdu),
        &AuthorizationRules::V7,
        factory.room_create_pdu(),
        factory.state_event_fn(),
    )
    .unwrap();
}

#[test]
fn knock_knock_join_rule_not_supported() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    factory.add_room_join_rules(
        owned_event_id!("$room-join-rules-knock"),
        UserFactory::Alice.user_id(),
        JoinRule::Knock,
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-zara-knock"),
        UserFactory::Zara.user_id(),
        RoomMemberPduContent::Knock,
    );

    // User CANNOT knock if room version doesn't support it.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V6,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "unknown membership"
    );
}

#[test]
fn knock_knock_restricted_join_rule() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V10);

    factory.add_room_join_rules(
        owned_event_id!("$room-join-rules-knock-restricted"),
        UserFactory::Alice.user_id(),
        JoinRule::KnockRestricted(Restricted::new(vec![AllowRule::RoomMembership(
            RoomMembership::new(owned_room_id!("!space:matrix.local")),
        )])),
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-zara-knock"),
        UserFactory::Zara.user_id(),
        RoomMemberPduContent::Knock,
    );

    // User can knock if room version supports it.
    check_room_member(
        RoomMemberEvent::new(pdu),
        &AuthorizationRules::V10,
        factory.room_create_pdu(),
        factory.state_event_fn(),
    )
    .unwrap();
}

#[test]
fn knock_knock_restricted_join_rule_not_supported() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    factory.add_room_join_rules(
        owned_event_id!("$room-join-rules-knock-restricted"),
        UserFactory::Alice.user_id(),
        JoinRule::KnockRestricted(Restricted::new(vec![AllowRule::RoomMembership(
            RoomMembership::new(owned_room_id!("!space:matrix.local")),
        )])),
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-zara-knock"),
        UserFactory::Zara.user_id(),
        RoomMemberPduContent::Knock,
    );

    // User CANNOT knock if room version doesn't support it.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V6,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "unknown membership"
    );
}

#[test]
fn knock_sender_state_key_mismatch() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V7);

    factory.add_room_join_rules(
        owned_event_id!("$room-join-rules-knock"),
        UserFactory::Alice.user_id(),
        JoinRule::Knock,
    );

    let mut pdu = factory.create_room_member(
        owned_event_id!("$room-member-zara-knock"),
        UserFactory::Zara.user_id(),
        RoomMemberPduContent::Knock,
    );
    pdu.sender = UserFactory::Bob.user_id();

    // User cannot knock if state key doesn't match sender.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V7,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "cannot make another user knock, sender does not match target user"
    );
}

#[test]
fn knock_after_ban() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V7);
    let alice_id = UserFactory::Alice.user_id();
    let bob_id = UserFactory::Bob.user_id();

    factory.add_room_join_rules(
        owned_event_id!("$room-join-rules-knock"),
        alice_id.clone(),
        JoinRule::Knock,
    );
    factory.add_room_member(
        owned_event_id!("$room-member-bob-ban"),
        bob_id.clone(),
        RoomMemberPduContent::Ban { sender: alice_id },
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member"),
        bob_id,
        RoomMemberPduContent::Knock,
    );

    // User cannot knock if banned.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V7,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "cannot knock if user is banned, invited or joined"
    );
}

#[test]
fn knock_after_invite() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V7);
    let zara_id = UserFactory::Zara.user_id();

    factory.add_room_join_rules(
        owned_event_id!("$room-join-rules-knock"),
        UserFactory::Alice.user_id(),
        JoinRule::Knock,
    );
    factory.add_room_member(
        owned_event_id!("$room-member-zara-invite"),
        zara_id.clone(),
        RoomMemberPduContent::Invite { sender: UserFactory::Bob.user_id() },
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-zara-knock"),
        zara_id,
        RoomMemberPduContent::Knock,
    );

    // User cannot knock after being invited.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V7,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "cannot knock if user is banned, invited or joined"
    );
}

#[test]
fn knock_after_join() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V7);

    factory.add_room_join_rules(
        owned_event_id!("$room-join-rules-knock"),
        UserFactory::Alice.user_id(),
        JoinRule::Knock,
    );

    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-bob-knock"),
        UserFactory::Bob.user_id(),
        RoomMemberPduContent::Knock,
    );

    // User cannot knock after joining the room.
    assert_eq!(
        check_room_member(
            RoomMemberEvent::new(pdu),
            &AuthorizationRules::V7,
            factory.room_create_pdu(),
            factory.state_event_fn(),
        )
        .unwrap_err(),
        "cannot knock if user is banned, invited or joined"
    );
}
