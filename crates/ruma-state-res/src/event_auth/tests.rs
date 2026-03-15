use js_int::int;
use ruma_common::{
    RoomVersionId, owned_event_id, owned_room_id,
    room_version_rules::{AuthorizationRules, RoomIdFormatVersion},
};
use ruma_events::TimelineEventType;
use serde_json::json;
use test_log::test;

mod room_power_levels;

use super::check_room_create;
use crate::{
    check_state_dependent_auth_rules, check_state_independent_auth_rules,
    event_auth::check_room_redaction,
    events::{RoomCreateEvent, RoomPowerLevelsEvent},
    test_utils::{
        Pdu, PublicChatInitialPdu, RoomCreatePduBuilder, RoomMemberPduContent,
        RoomPowerLevelsPduContent, RoomTimelineFactory, UserFactory, default_room_id,
    },
};

#[test]
fn valid_room_create() {
    let alice_id = UserFactory::Alice.user_id();

    // In room v1-v10, `m.room.create` event must have a `creator`.
    let mut pdu = RoomCreatePduBuilder::new(RoomVersionId::V6).build();
    check_room_create(RoomCreateEvent::new(&pdu), &AuthorizationRules::V6).unwrap();

    // Only keep the required `creator` field, which means the room version is the default of v1.
    pdu.set_content(json!({ "creator": alice_id }));
    check_room_create(RoomCreateEvent::new(&pdu), &AuthorizationRules::V1).unwrap();

    // Since room v11, we don't have the `creator` field.
    let mut pdu = RoomCreatePduBuilder::new(RoomVersionId::V11).build();
    check_room_create(RoomCreateEvent::new(&pdu), &AuthorizationRules::V11).unwrap();

    // Check various contents that might not match the definition of `m.room.create` in the
    // spec, to ensure that we only care about a few fields.
    let contents_to_check = vec![
        // With an invalid predecessor, but we don't care about it. Inspired by a real-life
        // example.
        json!({
            "room_version": "11",
            "predecessor": "!XPoLiaavxVgyMSiRwK:localhost",
        }),
        // With an invalid type, but we don't care about it.
        json!({
            "room_version": "11",
            "type": true,
        }),
    ];

    for content in contents_to_check {
        pdu.set_content(content);
        check_room_create(RoomCreateEvent::new(&pdu), &AuthorizationRules::V11).unwrap();
    }

    // In room v1-v11, check that `additional_creators` is not checked.
    pdu.set_content(json!({
        "room_version": "11",
        "additional_creators": ["@::example.org"]
    }));
    check_room_create(RoomCreateEvent::new(&pdu), &AuthorizationRules::V11).unwrap();

    // Since room v12, check that `additional_creators` only contains valid user IDs.
    let pdu =
        RoomCreatePduBuilder::new(RoomVersionId::V12).additional_creators(vec![alice_id]).build();
    check_room_create(RoomCreateEvent::new(pdu), &AuthorizationRules::V12).unwrap();
}

#[test]
fn invalid_room_create() {
    let valid_v6_pdu = RoomCreatePduBuilder::new(RoomVersionId::V6).build();
    let valid_v12_pdu = RoomCreatePduBuilder::new(RoomVersionId::V12).build();

    // `m.room.create` cannot have a prev event.
    let mut pdu = valid_v6_pdu.clone();
    pdu.prev_events.insert(owned_event_id!("$other-room-create"));
    assert_eq!(
        check_room_create(RoomCreateEvent::new(pdu), &AuthorizationRules::V6).unwrap_err(),
        "`m.room.create` event cannot have previous events"
    );

    // In room v1-v11, the sender must have the same server name as the room ID.
    let mut pdu = valid_v6_pdu.clone();
    let zara_id = UserFactory::Zara.user_id();
    pdu.set_content(json!({
        "creator": zara_id,
        "room_version": "6",
    }));
    pdu.sender = zara_id;
    assert_eq!(
        check_room_create(RoomCreateEvent::new(pdu), &AuthorizationRules::V6).unwrap_err(),
        "invalid `room_id` field in `m.room.create` event: server name does not match sender's server name"
    );

    // In room v1-v10, there must be a `creator` field in the content.
    let mut pdu = valid_v6_pdu;
    pdu.set_content(json!({ "room_version": "6" }));
    assert_eq!(
        check_room_create(RoomCreateEvent::new(pdu), &AuthorizationRules::V6).unwrap_err(),
        "missing `creator` field in `m.room.create` event"
    );

    // Since room v12, the `room_id` field is forbidden.
    let mut pdu = valid_v12_pdu.clone();
    pdu.room_id = Some(owned_room_id!("!room:matrix.local"));
    assert_eq!(
        check_room_create(RoomCreateEvent::new(pdu), &AuthorizationRules::V12).unwrap_err(),
        "`m.room.create` event cannot have a `room_id` field"
    );

    // Since room v12, `additional_creators` must only contains valid user IDs.
    let mut pdu = valid_v12_pdu;
    pdu.set_content(json!({
        "room_version": "12",
        "additional_creators": ["@::example.org"]
    }));
    assert_eq!(
        check_room_create(RoomCreateEvent::new(pdu), &AuthorizationRules::V12).unwrap_err(),
        "invalid `additional_creators` field in `m.room.create` event: server name is not a valid IP address or domain name at line 1 column 41"
    );
}

#[test]
fn redact_higher_power_level() {
    // The `m.room.redaction` checks are only done on room v1-v2 which are not supported by
    // RoomTimelineFactory, so construct the PDUs manually.
    let alice_id = UserFactory::Alice.user_id();

    let mut room_redaction_event = Pdu::with_minimal_fields(
        owned_event_id!("$redaction:matrix.local"),
        alice_id.clone(),
        TimelineEventType::RoomRedaction,
        json!({}),
    );
    // The redacted event ID must use another server name.
    room_redaction_event.redacts = Some(owned_event_id!("$other-event:other.local"));

    let room_power_levels_event = Pdu::with_minimal_state_fields(
        owned_event_id!("$powerlevels:matrix.local"),
        alice_id.clone(),
        TimelineEventType::RoomPowerLevels,
        String::new(),
        // All users have the default power level.
        json!({}),
    );

    // In room v1-v2, we cannot redact if redact level is higher than user's.
    assert_eq!(
        check_room_redaction(
            room_redaction_event,
            Some(RoomPowerLevelsEvent::new(room_power_levels_event)),
            &AuthorizationRules::V1,
            int!(0).into(),
        )
        .unwrap_err(),
        "`m.room.redaction` event did not pass any of the allow rules"
    );
}

#[test]
fn redact_same_power_level() {
    // The `m.room.redaction` checks are only done on room v1-v2 which are not supported by
    // RoomTimelineFactory, so construct the PDUs manually.
    let alice_id = UserFactory::Alice.user_id();

    let mut room_redaction_event = Pdu::with_minimal_fields(
        owned_event_id!("$redaction:matrix.local"),
        alice_id.clone(),
        TimelineEventType::RoomRedaction,
        json!({}),
    );
    // The redacted event ID must use another server name.
    room_redaction_event.redacts = Some(owned_event_id!("$other-event:other.local"));

    let room_power_levels_event = Pdu::with_minimal_state_fields(
        owned_event_id!("$powerlevels:matrix.local"),
        alice_id.clone(),
        TimelineEventType::RoomPowerLevels,
        String::new(),
        // This user is a moderator.
        json!({ "users": { alice_id: 50 }}),
    );

    // In room v1-v2, we can redact if redact level is same as user's.
    check_room_redaction(
        room_redaction_event,
        Some(RoomPowerLevelsEvent::new(room_power_levels_event)),
        &AuthorizationRules::V1,
        int!(50).into(),
    )
    .unwrap();
}

#[test]
fn redact_same_server() {
    // The `m.room.redaction` checks are only done on room v1-v2 which are not supported by
    // RoomTimelineFactory, so construct the PDUs manually.
    let alice_id = UserFactory::Alice.user_id();

    let mut room_redaction_event = Pdu::with_minimal_fields(
        owned_event_id!("$redaction:matrix.local"),
        alice_id.clone(),
        TimelineEventType::RoomRedaction,
        json!({}),
    );
    // The redacted event ID must use the same server name.
    room_redaction_event.redacts = Some(owned_event_id!("$other-event:matrix.local"));

    let room_power_levels_event = Pdu::with_minimal_state_fields(
        owned_event_id!("$powerlevels:matrix.local"),
        alice_id.clone(),
        TimelineEventType::RoomPowerLevels,
        String::new(),
        // All users have the default power level of `0`.
        json!({}),
    );

    // In room v1-v2, we can redact if redact level is same as user's.
    check_room_redaction(
        room_redaction_event,
        Some(RoomPowerLevelsEvent::new(room_power_levels_event)),
        &AuthorizationRules::V1,
        int!(0).into(),
    )
    .unwrap();
}

#[test]
fn reject_missing_room_create_auth_events() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    let mut pdu = factory.create_text_message(
        owned_event_id!("$hello"),
        UserFactory::Alice.user_id(),
        "Hello!",
    );
    pdu.auth_events.remove(&PublicChatInitialPdu::RoomCreate.event_id());

    // In room v1-v11, we cannot accept event if no `m.room.create` in auth events.
    assert_eq!(
        check_state_independent_auth_rules(&AuthorizationRules::V6, pdu, factory.get_fn())
            .unwrap_err(),
        "no `m.room.create` event in auth events"
    );
}

#[test]
fn no_federate() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);
    factory.get_mut(&PublicChatInitialPdu::RoomCreate.event_id()).unwrap().set_content(json!({
        "creator": UserFactory::Alice.user_id(),
        "m.federate": false,
    }));

    // Cannot accept event if not federating and different server.
    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-zara-join"),
        UserFactory::Zara.user_id(),
        RoomMemberPduContent::Join,
    );

    assert_eq!(
        check_state_dependent_auth_rules(&AuthorizationRules::V6, pdu, factory.state_event_fn())
            .unwrap_err(),
        "room is not federated and event's sender domain does not match `m.room.create` event's sender domain"
    );

    // Accept event if not federating and same server.
    let pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-join"),
        UserFactory::Charlie.user_id(),
        RoomMemberPduContent::Join,
    );

    check_state_dependent_auth_rules(&AuthorizationRules::V6, pdu, factory.state_event_fn())
        .unwrap();
}

#[test]
fn room_aliases_no_state_key() {
    // The event format didn't change between v4 and v6 so let's just create a v6 room.
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    let mut pdu = Pdu::with_minimal_fields(
        owned_event_id!("$room-aliases"),
        UserFactory::Alice.user_id(),
        TimelineEventType::RoomAliases,
        json!({
            "aliases": [
                "#alias:matrix.local",
                "#other_alias:matrix.local",
            ],
        }),
    );
    factory.prepare_to_add_pdu(&mut pdu);

    // In room v1-v5, cannot accept `m.room.aliases` without state key.
    assert_eq!(
        check_state_dependent_auth_rules(&AuthorizationRules::V3, &pdu, factory.state_event_fn())
            .unwrap_err(),
        "server name of the `state_key` of `m.room.aliases` event does not match the server name of the sender"
    );

    // Since room v6, `m.room.aliases` is not checked.
    check_state_dependent_auth_rules(&AuthorizationRules::V6, &pdu, factory.state_event_fn())
        .unwrap();
}

#[test]
fn room_aliases_other_server() {
    // The event format didn't change between v4 and v6 so let's just create a v6 room.
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    let mut pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-aliases"),
        UserFactory::Alice.user_id(),
        TimelineEventType::RoomAliases,
        "other.local".to_owned(),
        json!({
            "aliases": [
                "#alias:matrix.local",
                "#other_alias:matrix.local",
            ],
        }),
    );
    factory.prepare_to_add_pdu(&mut pdu);

    // In room v1-v5, we cannot accept `m.room.aliases` with different state key server name than
    // sender.
    assert_eq!(
        check_state_dependent_auth_rules(&AuthorizationRules::V3, &pdu, factory.state_event_fn())
            .unwrap_err(),
        "server name of the `state_key` of `m.room.aliases` event does not match the server name of the sender"
    );

    // Since room v6, `m.room.aliases` is not checked.
    check_state_dependent_auth_rules(&AuthorizationRules::V8, &pdu, factory.state_event_fn())
        .unwrap();
}

#[test]
fn room_aliases_same_server() {
    // The event format didn't change between v4 and v6 so let's just create a v6 room.
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    let mut pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-aliases"),
        UserFactory::Alice.user_id(),
        TimelineEventType::RoomAliases,
        "matrix.local".to_owned(),
        json!({
            "aliases": [
                "#alias:matrix.local",
                "#other_alias:matrix.local",
            ],
        }),
    );
    factory.prepare_to_add_pdu(&mut pdu);

    // Accept `m.room.aliases` with same server name as sender.
    check_state_dependent_auth_rules(&AuthorizationRules::V3, &pdu, factory.state_event_fn())
        .unwrap();

    // `m.room.aliases` is not checked since v6.
    check_state_dependent_auth_rules(&AuthorizationRules::V8, &pdu, factory.state_event_fn())
        .unwrap();
}

#[test]
fn sender_not_in_room() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    let pdu = factory.create_text_message(
        owned_event_id!("$hello"),
        UserFactory::Charlie.user_id(),
        "Hello!",
    );

    // Cannot accept event if user not in room.
    assert_eq!(
        check_state_dependent_auth_rules(&AuthorizationRules::V6, pdu, factory.state_event_fn())
            .unwrap_err(),
        "sender's membership is not `join`"
    );
}

#[test]
fn room_third_party_invite() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    let mut pdu = factory.create_room_third_party_invite();

    // Accept `m.room.third_party_invite` if enough power to invite.
    check_state_dependent_auth_rules(&AuthorizationRules::V6, &pdu, factory.state_event_fn())
        .unwrap();

    // Increase the power level required to invite to 50.
    factory.add_room_power_levels(
        owned_event_id!("$room-power-levels-invite"),
        UserFactory::Alice.user_id(),
        RoomPowerLevelsPduContent::Invite { value: 50 },
    );
    factory.prepare_to_add_pdu(&mut pdu);

    // Cannot accept `m.room.third_party_invite` if not enough power to invite.
    assert_eq!(
        check_state_dependent_auth_rules(&AuthorizationRules::V6, &pdu, factory.state_event_fn())
            .unwrap_err(),
        "sender does not have enough power to send invites in this room"
    );
}

#[test]
fn event_type_not_enough_power() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    let mut pdu = factory.create_text_message(
        owned_event_id!("$hello"),
        UserFactory::Bob.user_id(),
        "Hello!",
    );

    // Accept event if enough power for the event's type.
    check_state_dependent_auth_rules(&AuthorizationRules::V6, &pdu, factory.state_event_fn())
        .unwrap();

    // Increase the power level required to send `m.room.message` events.
    let alice_id = UserFactory::Alice.user_id();
    factory.add_room_power_levels(
        owned_event_id!("$room-power-levels-invite"),
        alice_id.clone(),
        RoomPowerLevelsPduContent::Events {
            event_types: vec![TimelineEventType::RoomMessage],
            value: 10,
        },
    );
    factory.prepare_to_add_pdu(&mut pdu);

    // Cannot send event if not enough power for the event's type.
    assert_eq!(
        check_state_dependent_auth_rules(&AuthorizationRules::V6, pdu, factory.state_event_fn())
            .unwrap_err(),
        "sender does not have enough power to send event of type `m.room.message`"
    );
}

#[test]
fn user_id_state_key_not_sender() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    let mut pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$fake-state-event"),
        UserFactory::Alice.user_id(),
        "dev.ruma.fake_state_event".into(),
        UserFactory::Zara.user_id().into(),
        json!({}),
    );
    factory.prepare_to_add_pdu(&mut pdu);

    // Cannot send state event with a user ID as a state key that doesn't match the sender.
    assert_eq!(
        check_state_dependent_auth_rules(&AuthorizationRules::V6, pdu, factory.state_event_fn())
            .unwrap_err(),
        "sender cannot send event with `state_key` matching another user's ID"
    );
}

#[test]
fn user_id_state_key_is_sender() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    let alice_id = UserFactory::Alice.user_id();
    let mut pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$fake-state-event"),
        alice_id.clone(),
        "dev.ruma.fake_state_event".into(),
        alice_id.into(),
        json!({}),
    );
    factory.prepare_to_add_pdu(&mut pdu);

    // Can send state event with a user ID as a state key that matches the sender.
    check_state_dependent_auth_rules(&AuthorizationRules::V6, pdu, factory.state_event_fn())
        .unwrap();
}

#[test]
fn auth_event_in_different_room() {
    let mut factory = RoomCreatePduBuilder::new(RoomVersionId::V6).build_factory();

    let mut pdu = factory.create_room_member(
        owned_event_id!("$room-member-alice-join"),
        UserFactory::Alice.user_id(),
        RoomMemberPduContent::Join,
    );
    // This is not the right room!
    pdu.room_id = Some(owned_room_id!("!wrongroom:matrix.local"));

    // Cannot accept with auth event in different room.
    assert_eq!(
        check_state_independent_auth_rules(&AuthorizationRules::V6, pdu, factory.get_fn())
            .unwrap_err(),
        "auth event $room-create not in the same room"
    );
}

#[test]
fn duplicate_auth_event_type() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    let alice_id = UserFactory::Alice.user_id();
    factory.add_room_member(
        owned_event_id!("$room-member-alice-displayname"),
        alice_id.clone(),
        RoomMemberPduContent::DisplayName { displayname: "Alice".to_owned() },
    );

    let mut pdu = factory.create_text_message(owned_event_id!("$hello"), alice_id, "Hello!");
    pdu.auth_events.insert(PublicChatInitialPdu::RoomMemberAliceJoin.event_id());

    // Cannot accept with two auth events with same (type, state_key) pair.
    assert_eq!(
        check_state_independent_auth_rules(&AuthorizationRules::V6, pdu, factory.get_fn())
            .unwrap_err(),
        "duplicate auth event $room-member-alice-join for (m.room.member, @alice:matrix.local) pair"
    );
}

#[test]
fn unexpected_auth_event_type() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    let mut pdu = factory.create_text_message(
        owned_event_id!("$hello"),
        UserFactory::Alice.user_id(),
        "Hello!",
    );
    pdu.auth_events.insert(PublicChatInitialPdu::RoomJoinRules.event_id());

    // Cannot accept with auth event with unexpected (type, state_key) pair.
    assert_eq!(
        check_state_independent_auth_rules(&AuthorizationRules::V6, pdu, factory.get_fn())
            .unwrap_err(),
        "unexpected auth event $room-join-rules with (m.room.join_rules, ) pair"
    );
}

#[test]
fn rejected_auth_event() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V6);

    let charlie_id = UserFactory::Charlie.user_id();
    let room_member_charlie_knock_pdu = factory.add_room_member(
        owned_event_id!("$room-member-charlie-knock"),
        charlie_id.clone(),
        RoomMemberPduContent::Knock,
    );
    // A user can't knock on a public room!.
    room_member_charlie_knock_pdu.rejected = true;

    // Bob's invite after the knock.
    let room_member_charlie_invite_pdu = factory.create_room_member(
        owned_event_id!("$room-member-charlie-invite"),
        charlie_id,
        RoomMemberPduContent::Invite { sender: UserFactory::Bob.user_id() },
    );

    // Cannot accept with auth event that was rejected.
    assert_eq!(
        check_state_independent_auth_rules(
            &AuthorizationRules::V6,
            room_member_charlie_invite_pdu,
            factory.get_fn(),
        )
        .unwrap_err(),
        "rejected auth event $room-member-charlie-knock"
    );
}

#[test]
fn room_create_with_allowed_or_rejected_room_id() {
    // The check ignores the `room_version` field in the content so we can use a PDU with the
    // wrong value regardless of the version of the authorization rules.

    // A room v11 PDU, with a room ID.
    let room_create_v11 = RoomCreatePduBuilder::new(RoomVersionId::V11).build();
    // A room v12 PDU, without a room ID.
    let room_create_v12 = RoomCreatePduBuilder::new(RoomVersionId::V12).build();

    check_room_create(RoomCreateEvent::new(&room_create_v11), &AuthorizationRules::V11).unwrap();

    // In room v1-v11, the room ID must be present.
    assert_eq!(
        check_room_create(RoomCreateEvent::new(&room_create_v12), &AuthorizationRules::V11)
            .unwrap_err(),
        "missing `room_id` field in `m.room.create` event"
    );

    // Since room v12, the room ID must not be present.
    assert_eq!(
        check_room_create(RoomCreateEvent::new(&room_create_v11), &AuthorizationRules::V12)
            .unwrap_err(),
        "`m.room.create` event cannot have a `room_id` field"
    );

    check_room_create(RoomCreateEvent::new(&room_create_v12), &AuthorizationRules::V12).unwrap();
}

#[test]
fn event_without_room_id() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V11);

    let mut pdu = factory.create_text_message(
        owned_event_id!("$hello"),
        UserFactory::Alice.user_id(),
        "Hello!",
    );
    pdu.room_id.take();

    // Cannot accept event without room ID.
    assert_eq!(
        check_state_independent_auth_rules(&AuthorizationRules::V11, pdu, factory.get_fn())
            .unwrap_err(),
        "missing `room_id` field for event"
    );
}

#[test]
fn allow_missing_room_create_auth_events() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V12);

    let pdu = factory.create_text_message(
        owned_event_id!("$hello"),
        UserFactory::Alice.user_id(),
        "Hello!",
    );
    assert!(!pdu.auth_events.contains(&PublicChatInitialPdu::RoomCreate.event_id()));

    // Since room v12, accept event if no `m.room.create` in auth events.
    check_state_independent_auth_rules(&AuthorizationRules::V12, pdu, factory.get_fn()).unwrap();
}

#[test]
fn reject_room_create_in_auth_events() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V12);

    let mut pdu = factory.create_text_message(
        owned_event_id!("$hello"),
        UserFactory::Alice.user_id(),
        "Hello!",
    );
    pdu.auth_events.insert(PublicChatInitialPdu::RoomCreate.event_id());

    // Since room v12, reject event if `m.room.create` in auth events.
    assert_eq!(
        check_state_independent_auth_rules(&AuthorizationRules::V12, pdu, factory.get_fn())
            .unwrap_err(),
        "auth event $room-create not in the same room"
    );
}

#[test]
fn missing_room_create_in_fetch_event() {
    let mut factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V12);

    let pdu = factory.create_text_message(
        owned_event_id!("$hello"),
        UserFactory::Alice.user_id(),
        "Hello!",
    );

    factory.remove(&PublicChatInitialPdu::RoomCreate.event_id());

    // Reject event if `m.room.create` can't be found.
    assert_eq!(
        check_state_independent_auth_rules(&AuthorizationRules::V12, pdu, factory.get_fn())
            .unwrap_err(),
        "failed to find `m.room.create` event $room-create"
    );
}

#[test]
fn rejected_room_create_in_fetch_event() {
    let mut room_create_pdu = RoomCreatePduBuilder::new(RoomVersionId::V12).build();
    room_create_pdu.rejected = true;
    let room_create_event_id = room_create_pdu.event_id.clone();

    let alice_id = UserFactory::Alice.user_id();
    let mut room_member_pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-member-alice-join"),
        alice_id.clone(),
        TimelineEventType::RoomMember,
        alice_id.into(),
        json!({ "membership": "join" }),
    );
    room_member_pdu.room_id = Some(default_room_id(&RoomIdFormatVersion::V2));
    room_member_pdu.prev_events.insert(room_create_event_id.clone());

    // Reject event if `m.room.create` was rejected.
    assert_eq!(
        check_state_independent_auth_rules(&AuthorizationRules::V12, room_member_pdu, |event_id| {
            (event_id == room_create_event_id).then_some(&room_create_pdu)
        })
        .unwrap_err(),
        "rejected `m.room.create` event $room-create"
    );
}
