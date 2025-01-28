use std::{borrow::Borrow, collections::BTreeSet};

use js_int::{int, Int};
use ruma_common::{
    serde::{Base64, Raw},
    OwnedUserId, RoomVersionId, UserId,
};
use ruma_events::room::{
    create::RoomCreateEventContent,
    join_rules::{JoinRule, RoomJoinRulesEventContent},
    member::{MembershipState, ThirdPartyInvite},
    power_levels::RoomPowerLevelsEventContent,
    third_party_invite::RoomThirdPartyInviteEventContent,
};
use serde::{
    de::{Error as _, IgnoredAny},
    Deserialize,
};
use serde_json::{from_str as from_json_str, value::RawValue as RawJsonValue};
use tracing::{debug, error, info, instrument, trace, warn};

use crate::{
    events::{
        deserialize_power_levels, deserialize_power_levels_content_fields,
        deserialize_power_levels_content_invite, deserialize_power_levels_content_redact,
    },
    room_version::RoomVersion,
    Error, Event, Result, StateEventType, TimelineEventType,
};

// TODO: We need methods for all checks performed on receipt of a PDU, plus the following that are
// not listed:
//
// - check that the event respects the size limits,
// - check that all the auth events are in the same room as `incoming_event`.
//
// References:
// - https://spec.matrix.org/latest/server-server-api/#checks-performed-on-receipt-of-a-pdu
// - https://spec.matrix.org/latest/client-server-api/#size-limits
// - https://github.com/element-hq/synapse/blob/9c5d08fff8d66a7cc0e2ecfeeb783f933a778c2f/synapse/event_auth.py
// - https://github.com/matrix-org/matrix-spec/issues/365

// FIXME: field extracting could be bundled for `content`
#[derive(Deserialize)]
struct GetMembership {
    membership: MembershipState,
}

#[derive(Deserialize)]
struct RoomMemberContentFields {
    membership: Option<Raw<MembershipState>>,
    join_authorised_via_users_server: Option<Raw<OwnedUserId>>,
}

/// Get the list of [relevant auth event types] required to authorize the event of the given type.
///
/// Returns a list of `(event_type, state_key)` tuples.
///
/// # Errors
///
/// Returns an error if `content` is not a JSON object.
///
/// [relevant auth events]: https://spec.matrix.org/latest/server-server-api/#auth-events-selection
pub fn auth_types_for_event(
    event_type: &TimelineEventType,
    sender: &UserId,
    state_key: Option<&str>,
    content: &RawJsonValue,
) -> serde_json::Result<Vec<(StateEventType, String)>> {
    // `m.room.create` is the first event in a room, it has no auth events.
    if event_type == &TimelineEventType::RoomCreate {
        return Ok(vec![]);
    }

    // All other events need these auth events.
    let mut auth_types = vec![
        (StateEventType::RoomPowerLevels, "".to_owned()),
        (StateEventType::RoomMember, sender.to_string()),
        (StateEventType::RoomCreate, "".to_owned()),
    ];

    // `m.room.member` events need other auth events.
    if event_type == &TimelineEventType::RoomMember {
        #[derive(Deserialize)]
        struct RoomMemberContentFields {
            membership: Option<Raw<MembershipState>>,
            third_party_invite: Option<Raw<ThirdPartyInvite>>,
            join_authorised_via_users_server: Option<Raw<OwnedUserId>>,
        }

        if let Some(state_key) = state_key {
            let content: RoomMemberContentFields = from_json_str(content.get())?;

            if let Some(Ok(membership)) = content.membership.map(|m| m.deserialize()) {
                if [MembershipState::Join, MembershipState::Invite, MembershipState::Knock]
                    .contains(&membership)
                {
                    // If membership is join, invite or knock, we need `m.room.join_rules`.
                    let key = (StateEventType::RoomJoinRules, "".to_owned());
                    if !auth_types.contains(&key) {
                        auth_types.push(key);
                    }

                    if let Some(Ok(u)) =
                        content.join_authorised_via_users_server.map(|m| m.deserialize())
                    {
                        // If `join_authorised_via_users_server` is present, and the room
                        // version supports restricted rooms, we need `m.room.member` with the
                        // matching state_key.
                        //
                        // FIXME: We need to check that the room version supports restricted rooms
                        // too.
                        let key = (StateEventType::RoomMember, u.to_string());
                        if !auth_types.contains(&key) {
                            auth_types.push(key);
                        }
                    }
                }

                let key = (StateEventType::RoomMember, state_key.to_owned());
                if !auth_types.contains(&key) {
                    auth_types.push(key);
                }

                if membership == MembershipState::Invite {
                    if let Some(Ok(t_id)) = content.third_party_invite.map(|t| t.deserialize()) {
                        // If membership is invite and `third_party_invite` is present, we need
                        // `m.room.third_party_invite` with the state_key matching
                        // `third_party_invite.signed.token`.
                        let key = (StateEventType::RoomThirdPartyInvite, t_id.signed.token);
                        if !auth_types.contains(&key) {
                            auth_types.push(key);
                        }
                    }
                }
            }
        }
    }

    Ok(auth_types)
}

/// Check whether the incoming event passes the [authorization rules] for the given room version.
///
/// The `fetch_state` closure should gather state from a state snapshot. We need to know if the
/// event passes auth against some state not a recursive collection of auth_events fields.
///
/// This assumes that `ruma_signatures::verify_event()` was called previously, as some authorization
/// rules depend on the signatures being valid on the event.
///
/// [authorization]: https://spec.matrix.org/latest/server-server-api/#authorization-rules
#[instrument(skip_all, fields(event_id = incoming_event.event_id().borrow().as_str()))]
pub fn auth_check<E: Event>(
    room_version: &RoomVersion,
    incoming_event: impl Event,
    current_third_party_invite: Option<impl Event>,
    fetch_state: impl Fn(&StateEventType, &str) -> Option<E>,
) -> Result<bool> {
    debug!("starting auth check");

    let sender = incoming_event.sender();

    // Since v1, if type is m.room.create:
    if *incoming_event.event_type() == TimelineEventType::RoomCreate {
        #[derive(Deserialize)]
        struct RoomCreateContentFields {
            room_version: Option<Raw<RoomVersionId>>,
            creator: Option<Raw<IgnoredAny>>,
        }

        debug!("start m.room.create check");

        // Since v1, if it has any previous events, reject.
        if incoming_event.prev_events().next().is_some() {
            warn!("the room creation event had previous events");
            return Ok(false);
        }

        // Since v1, if the domain of the room_id does not match the domain of the sender, reject.
        let Some(room_id_server_name) = incoming_event.room_id().server_name() else {
            warn!("room ID has no servername");
            return Ok(false);
        };

        if room_id_server_name != sender.server_name() {
            warn!("servername of room ID does not match servername of sender");
            return Ok(false);
        }

        // Since v1, if `content.room_version` is present and is not a recognized version, reject.
        //
        // FIXME: this only checks if we can deserialize to `RoomVersionId` which accepts any
        // string. We should check if the version is actually supported, i.e. if we have a
        // `RoomVersion` for it. But we already take a `RoomVersion` as a parameter so this was
        // already checked before?
        let content: RoomCreateContentFields = from_json_str(incoming_event.content().get())?;
        if content.room_version.map(|v| v.deserialize().is_err()).unwrap_or(false) {
            warn!("invalid room version found in m.room.create event");
            return Ok(false);
        }

        if !room_version.use_room_create_sender {
            // v1-v10, if content has no creator field, reject.
            if content.creator.is_none() {
                warn!("no creator field found in m.room.create content");
                return Ok(false);
            }
        }

        info!("m.room.create event was allowed");
        return Ok(true);
    }

    /*
    TODO: In the past this code caused problems federating with synapse, maybe this has been
    resolved already. Needs testing.

    // Since v1, considering auth_events:
    //
    // - Since v1, if there are duplicate entries for a given type and state_key pair, reject.
    //
    // - Since v1, if there are entries whose type and state_key don’t match those specified
    //   by the auth events selection algorithm described in the server specification, reject.
    let expected_auth = auth_types_for_event(
        incoming_event.kind,
        sender,
        incoming_event.state_key,
        incoming_event.content().clone(),
    );

    dbg!(&expected_auth);

    for ev_key in auth_events.keys() {
        // (b)
        if !expected_auth.contains(ev_key) {
            warn!("auth_events contained invalid auth event");
            return Ok(false);
        }
    }
    */

    // TODO:
    //
    // Since v1, if there are entries which were themselves rejected under the checks performed on
    // receipt of a PDU, reject.

    let room_create_event = match fetch_state(&StateEventType::RoomCreate, "") {
        None => {
            warn!("no m.room.create event in auth chain");
            return Ok(false);
        }
        Some(e) => e,
    };

    // Since v1, if there is no m.room.create event among the entries, reject.
    if !incoming_event.auth_events().any(|id| id.borrow() == room_create_event.event_id().borrow())
    {
        warn!("no m.room.create event in auth events");
        return Ok(false);
    }

    // Since v1, if the create event content has the field m.federate set to false and the sender
    // domain of the event does not match the sender domain of the create event, reject.
    #[derive(Deserialize)]
    struct RoomCreateContentFederate {
        #[serde(rename = "m.federate", default = "ruma_common::serde::default_true")]
        federate: bool,
    }
    let room_create_content: RoomCreateContentFederate =
        from_json_str(room_create_event.content().get())?;
    if !room_create_content.federate
        && room_create_event.sender().server_name() != incoming_event.sender().server_name()
    {
        warn!("room is not federated and event's sender domain does not match create event's sender domain");
        return Ok(false);
    }

    // v1-v5, if type is m.room.aliases:
    if room_version.special_case_aliases_auth
        && *incoming_event.event_type() == TimelineEventType::RoomAliases
    {
        debug!("starting m.room.aliases check");
        // v1-v5, if event has no state_key, reject.
        //
        // v1-v5, if sender's domain doesn't match state_key, reject.
        if incoming_event.state_key() != Some(sender.server_name().as_str()) {
            warn!("state_key does not match sender");
            return Ok(false);
        }

        // Otherwise, allow.
        info!("m.room.aliases event was allowed");
        return Ok(true);
    }

    let power_levels_event = fetch_state(&StateEventType::RoomPowerLevels, "");
    let sender_member_event = fetch_state(&StateEventType::RoomMember, sender.as_str());

    // Since v1, if type is m.room.member:
    if *incoming_event.event_type() == TimelineEventType::RoomMember {
        debug!("starting m.room.member check");
        // Since v1, if there is no state_key property, or no membership property in content,
        // reject.
        let state_key = match incoming_event.state_key() {
            None => {
                warn!("no statekey in member event");
                return Ok(false);
            }
            Some(s) => s,
        };

        let content: RoomMemberContentFields = from_json_str(incoming_event.content().get())?;
        if content.membership.as_ref().and_then(|m| m.deserialize().ok()).is_none() {
            warn!("no valid membership field found for m.room.member event content");
            return Ok(false);
        }

        let target_user =
            <&UserId>::try_from(state_key).map_err(|e| Error::InvalidPdu(format!("{e}")))?;

        let user_for_join_auth =
            content.join_authorised_via_users_server.as_ref().and_then(|u| u.deserialize().ok());

        let user_for_join_auth_membership = user_for_join_auth
            .as_ref()
            .and_then(|auth_user| fetch_state(&StateEventType::RoomMember, auth_user.as_str()))
            .and_then(|mem| from_json_str::<GetMembership>(mem.content().get()).ok())
            .map(|mem| mem.membership)
            .unwrap_or(MembershipState::Leave);

        if !valid_membership_change(
            room_version,
            target_user,
            fetch_state(&StateEventType::RoomMember, target_user.as_str()).as_ref(),
            sender,
            sender_member_event.as_ref(),
            &incoming_event,
            current_third_party_invite,
            power_levels_event.as_ref(),
            fetch_state(&StateEventType::RoomJoinRules, "").as_ref(),
            user_for_join_auth.as_deref(),
            &user_for_join_auth_membership,
            room_create_event,
        )? {
            return Ok(false);
        }

        info!("m.room.member event was allowed");
        return Ok(true);
    }

    // Since v1, if the sender's current membership state is not join, reject.
    let sender_member_event = match sender_member_event {
        Some(mem) => mem,
        None => {
            warn!("sender not found in room");
            return Ok(false);
        }
    };

    let sender_membership_event_content: RoomMemberContentFields =
        from_json_str(sender_member_event.content().get())?;
    let membership_state = sender_membership_event_content
        .membership
        .expect("we should test before that this field exists")
        .deserialize()?;

    if !matches!(membership_state, MembershipState::Join) {
        warn!("sender's membership is not join");
        return Ok(false);
    }

    let sender_power_level = if let Some(pl) = &power_levels_event {
        let content = deserialize_power_levels_content_fields(pl.content().get(), room_version)?;
        if let Some(level) = content.users.get(sender) {
            *level
        } else {
            content.users_default
        }
    } else {
        // If no power level event found the creator gets 100 everyone else gets 0
        let is_creator = if room_version.use_room_create_sender {
            room_create_event.sender() == sender
        } else {
            #[allow(deprecated)]
            from_json_str::<RoomCreateEventContent>(room_create_event.content().get())
                .is_ok_and(|create| create.creator.unwrap() == *sender)
        };

        if is_creator {
            int!(100)
        } else {
            int!(0)
        }
    };

    // Since v1, if type is m.room.third_party_invite:
    if *incoming_event.event_type() == TimelineEventType::RoomThirdPartyInvite {
        // Since v1, allow if and only if sender's current power level is greater than
        // or equal to the invite level.
        let invite_level = match &power_levels_event {
            Some(power_levels) => {
                deserialize_power_levels_content_invite(power_levels.content().get(), room_version)?
                    .invite
            }
            None => int!(0),
        };

        if sender_power_level < invite_level {
            warn!("sender cannot send invites in this room");
            return Ok(false);
        }

        info!("m.room.third_party_invite event was allowed");
        return Ok(true);
    }

    if !can_send_event(&incoming_event, power_levels_event.as_ref(), sender_power_level) {
        warn!("user cannot send event");
        return Ok(false);
    }

    // If type is m.room.power_levels
    if *incoming_event.event_type() == TimelineEventType::RoomPowerLevels {
        debug!("starting m.room.power_levels check");

        if let Some(required_pwr_lvl) = check_power_levels(
            room_version,
            &incoming_event,
            power_levels_event.as_ref(),
            sender_power_level,
        ) {
            if !required_pwr_lvl {
                warn!("m.room.power_levels was not allowed");
                return Ok(false);
            }
        } else {
            warn!("m.room.power_levels was not allowed");
            return Ok(false);
        }
        info!("m.room.power_levels event allowed");
    }

    // v1-v2, if type is m.room.redaction:
    if room_version.extra_redaction_checks
        && *incoming_event.event_type() == TimelineEventType::RoomRedaction
    {
        let redact_level = match power_levels_event {
            Some(pl) => {
                deserialize_power_levels_content_redact(pl.content().get(), room_version)?.redact
            }
            None => int!(50),
        };

        if !check_redaction(room_version, incoming_event, sender_power_level, redact_level)? {
            return Ok(false);
        }
    }

    // Otherwise, allow.
    info!("allowing event passed all checks");
    Ok(true)
}

// TODO deserializing the member, power, join_rules event contents is done in conduit
// just before this is called. Could they be passed in?
/// Check if the `m.room.member` event with the given properties passes the authorization rules
/// specific to its event type.
///
/// This assumes that `ruma_signatures::verify_event()` was called previously, as some authorization
/// rules depend on the signatures being valid on the event.
#[allow(clippy::too_many_arguments)]
fn valid_membership_change(
    room_version: &RoomVersion,
    target_user: &UserId,
    target_user_membership_event: Option<impl Event>,
    sender: &UserId,
    sender_membership_event: Option<impl Event>,
    current_event: impl Event,
    current_third_party_invite: Option<impl Event>,
    power_levels_event: Option<impl Event>,
    join_rules_event: Option<impl Event>,
    user_for_join_auth: Option<&UserId>,
    user_for_join_auth_membership: &MembershipState,
    create_room: impl Event,
) -> Result<bool> {
    #[derive(Deserialize)]
    struct GetThirdPartyInvite {
        third_party_invite: Option<Raw<ThirdPartyInvite>>,
    }
    let content = current_event.content();

    let target_membership = from_json_str::<GetMembership>(content.get())?.membership;
    let third_party_invite =
        from_json_str::<GetThirdPartyInvite>(content.get())?.third_party_invite;

    let sender_membership = match &sender_membership_event {
        Some(pdu) => from_json_str::<GetMembership>(pdu.content().get())?.membership,
        None => MembershipState::Leave,
    };
    let sender_is_joined = sender_membership == MembershipState::Join;

    let target_user_current_membership = match &target_user_membership_event {
        Some(pdu) => from_json_str::<GetMembership>(pdu.content().get())?.membership,
        None => MembershipState::Leave,
    };

    let power_levels: RoomPowerLevelsEventContent = match &power_levels_event {
        Some(ev) => from_json_str(ev.content().get())?,
        None => RoomPowerLevelsEventContent::default(),
    };

    // FIXME: Do we really need to check if the sender is joined to get their power level? The auth
    // rules below already check it.
    let sender_power = power_levels
        .users
        .get(sender)
        .or_else(|| sender_is_joined.then_some(&power_levels.users_default));

    let target_power = power_levels.users.get(target_user).or_else(|| {
        (target_membership == MembershipState::Join).then_some(&power_levels.users_default)
    });

    let mut join_rules = JoinRule::Invite;
    if let Some(jr) = &join_rules_event {
        join_rules = from_json_str::<RoomJoinRulesEventContent>(jr.content().get())?.join_rule;
    }

    let power_levels_event_id = power_levels_event.as_ref().map(|e| e.event_id());
    let sender_membership_event_id = sender_membership_event.as_ref().map(|e| e.event_id());
    let target_user_membership_event_id =
        target_user_membership_event.as_ref().map(|e| e.event_id());

    let user_for_join_auth_is_valid = if let Some(user_for_join_auth) = user_for_join_auth {
        // Is the authorised user allowed to invite users into this room
        let (auth_user_pl, invite_level) = if let Some(pl) = &power_levels_event {
            // TODO Refactor all powerlevel parsing
            let invite =
                deserialize_power_levels_content_invite(pl.content().get(), room_version)?.invite;

            let content =
                deserialize_power_levels_content_fields(pl.content().get(), room_version)?;
            let user_pl = if let Some(level) = content.users.get(user_for_join_auth) {
                *level
            } else {
                content.users_default
            };

            (user_pl, invite)
        } else {
            (int!(0), int!(0))
        };
        (user_for_join_auth_membership == &MembershipState::Join) && (auth_user_pl >= invite_level)
    } else {
        // No auth user was given
        false
    };

    // These checks are done `in ruma_signatures::verify_event()`:
    //
    // Since v8, if content has a join_authorised_via_users_server property:
    //
    // - Since v8, if the event is not validly signed by the homeserver of the user ID denoted by
    //   the key, reject.

    Ok(match target_membership {
        // Since v1, if membership is join:
        MembershipState::Join => {
            // v1-v10, if the only previous event is an m.room.create and the state_key is the
            // creator, allow.
            // Since v11, if the only previous event is an m.room.create and the state_key is the
            // sender of the m.room.create, allow.
            let mut prev_events = current_event.prev_events();

            let prev_event_is_create_event = prev_events
                .next()
                .map(|event_id| event_id.borrow() == create_room.event_id().borrow())
                .unwrap_or(false);
            let no_more_prev_events = prev_events.next().is_none();

            if prev_event_is_create_event && no_more_prev_events {
                let is_creator = if room_version.use_room_create_sender {
                    let creator = create_room.sender();

                    creator == sender && creator == target_user
                } else {
                    #[allow(deprecated)]
                    let creator =
                        from_json_str::<RoomCreateEventContent>(create_room.content().get())?
                            .creator
                            .ok_or_else(|| serde_json::Error::missing_field("creator"))?;

                    creator == sender && creator == target_user
                };

                if is_creator {
                    return Ok(true);
                }
            }

            if sender != target_user {
                // Since v1, if the sender does not match state_key, reject.
                warn!("can't make other user join");
                false
            } else if let MembershipState::Ban = target_user_current_membership {
                // Since v1, if the sender is banned, reject.
                warn!(?target_user_membership_event_id, "banned user can't join");
                false
            } else if (join_rules == JoinRule::Invite
                || room_version.allow_knocking && join_rules == JoinRule::Knock)
                && (target_user_current_membership == MembershipState::Join
                    || target_user_current_membership == MembershipState::Invite)
            {
                // v1-v6, if the join_rule is invite then allow if membership state is invite or
                // join.
                // Since v7, if the join_rule is invite or knock then allow if membership state is
                // invite or join.
                true
            } else if room_version.restricted_join_rules
                && matches!(join_rules, JoinRule::Restricted(_))
                || room_version.knock_restricted_join_rule
                    && matches!(join_rules, JoinRule::KnockRestricted(_))
            {
                // v8-v9, if the join_rule is restricted:
                // Since v10, if the join_rule is restricted or knock_restricted:
                if matches!(
                    target_user_current_membership,
                    MembershipState::Invite | MembershipState::Join
                ) {
                    // Since v8, if membership state is join or invite, allow.
                    true
                } else {
                    // Since v8, if the join_authorised_via_users_server key in content is not a
                    // user with sufficient permission to invite other users, reject.
                    //
                    // Otherwise, allow.
                    user_for_join_auth_is_valid
                }
            } else {
                // Since v1, if the join_rule is public, allow.
                // Otherwise, reject.
                join_rules == JoinRule::Public
            }
        }
        // Since v1, if membership is invite:
        MembershipState::Invite => {
            if let Some(tp_id) = third_party_invite.and_then(|i| i.deserialize().ok()) {
                // Since v1, if content has a third_party_invite property:
                if target_user_current_membership == MembershipState::Ban {
                    // Since v1, if target user is banned, reject.
                    warn!(?target_user_membership_event_id, "can't invite banned user");
                    false
                } else {
                    let allow = verify_third_party_invite(
                        Some(target_user),
                        sender,
                        &tp_id,
                        current_third_party_invite,
                    );
                    if !allow {
                        warn!("third party invite invalid");
                    }
                    allow
                }
            } else if !sender_is_joined
                || target_user_current_membership == MembershipState::Join
                || target_user_current_membership == MembershipState::Ban
            {
                // Since v1, if the sender’s current membership state is not join, reject.
                //
                // Since v1, if target user’s current membership state is join or ban, reject.
                warn!(
                    ?target_user_membership_event_id,
                    ?sender_membership_event_id,
                    "can't invite user if sender not joined or the user is currently joined or \
                     banned",
                );
                false
            } else {
                // Since v1, if the sender’s power level is greater than or equal to the invite
                // level, allow.
                //
                // Otherwise, reject.
                let allow = sender_power.filter(|&p| p >= &power_levels.invite).is_some();
                if !allow {
                    warn!(
                        ?target_user_membership_event_id,
                        ?power_levels_event_id,
                        "user does not have enough power to invite",
                    );
                }
                allow
            }
        }
        // Since v1, if membership is leave:
        MembershipState::Leave => {
            if sender == target_user {
                // v1-v6, if the sender matches state_key, allow if and only if that user’s current
                // membership state is invite or join. Since v7, if the sender
                // matches state_key, allow if and only if that user’s current membership state is
                // invite, join, or knock.
                //
                // FIXME: This does not check for knock membership.
                let allow = target_user_current_membership == MembershipState::Join
                    || target_user_current_membership == MembershipState::Invite;
                if !allow {
                    warn!(?target_user_membership_event_id, "can't leave if not invited or joined");
                }
                allow
            } else if !sender_is_joined
                || target_user_current_membership == MembershipState::Ban
                    && sender_power.filter(|&p| p < &power_levels.ban).is_some()
            {
                // Since v1, if the sender’s current membership state is not join, reject.
                //
                // Since v1, if the target user’s current membership state is ban, and the sender’s
                // power level is less than the ban level, reject.
                warn!(
                    ?target_user_membership_event_id,
                    ?sender_membership_event_id,
                    "can't kick if sender not joined or user is already banned",
                );
                false
            } else {
                // Since v1, if the sender’s power level is greater than or equal to the kick level,
                // and the target user’s power level is less than the sender’s power level, allow.
                //
                // Otherwise, reject.
                let allow = sender_power.filter(|&p| p >= &power_levels.kick).is_some()
                    && target_power < sender_power;
                if !allow {
                    warn!(
                        ?target_user_membership_event_id,
                        ?power_levels_event_id,
                        "user does not have enough power to kick",
                    );
                }
                allow
            }
        }
        // Since v1, if membership is ban:
        MembershipState::Ban => {
            if !sender_is_joined {
                // Since v1, if the sender’s current membership state is not join, reject.
                warn!(?sender_membership_event_id, "can't ban user if sender is not joined");
                false
            } else {
                // If the sender’s power level is greater than or equal to the ban level, and the
                // target user’s power level is less than the sender’s power level, allow.
                //
                // Otherwise, reject.
                let allow = sender_power.filter(|&p| p >= &power_levels.ban).is_some()
                    && target_power < sender_power;
                if !allow {
                    warn!(
                        ?target_user_membership_event_id,
                        ?power_levels_event_id,
                        "user does not have enough power to ban",
                    );
                }
                allow
            }
        }
        // Since v7, if membership is knock:
        MembershipState::Knock if room_version.allow_knocking => {
            if join_rules != JoinRule::Knock
                || room_version.knock_restricted_join_rule
                    && matches!(join_rules, JoinRule::KnockRestricted(_))
            {
                // v7-v9, if the join_rule is anything other than knock, reject.
                // Since v10, if the join_rule is anything other than knock or knock_restricted,
                // reject.
                warn!("join rule is not set to knock or knock_restricted, knocking is not allowed");
                false
            } else if sender != target_user {
                // Since v7, if sender does not match state_key, reject.
                warn!(
                    ?sender,
                    ?target_user,
                    "can't make another user knock, sender did not match target"
                );
                false
            } else if matches!(
                sender_membership,
                MembershipState::Ban | MembershipState::Invite | MembershipState::Join
            ) {
                // Since v7, if the sender’s current membership is not ban, invite, or join, allow.
                // Otherwise, reject.
                warn!(
                    ?target_user_membership_event_id,
                    "membership state of ban, invite or join are invalid",
                );
                false
            } else {
                true
            }
        }
        // Since v1, otherwise, the membership is unknown. Reject.
        _ => {
            warn!("unknown membership transition");
            false
        }
    })
}

/// Check if the user is allowed to send the given event.
///
/// Criteria:
///
/// - The user is allowed based on the required event type's room power level.
/// - If there is a state key that is a user ID, it must be the same as the sender.
fn can_send_event(event: impl Event, ple: Option<impl Event>, user_level: Int) -> bool {
    // Since v1, if the event type's required power level is greater than the sender's power level,
    // reject.
    let event_type_power_level = get_send_level(event.event_type(), event.state_key(), ple);

    debug!(
        required_level = i64::from(event_type_power_level),
        user_level = i64::from(user_level),
        state_key = ?event.state_key(),
        "permissions factors",
    );

    if user_level < event_type_power_level {
        return false;
    }

    // Since v1, if the event has a state_key that starts with an @ and does not match the sender,
    // reject.
    if event.state_key().is_some_and(|k| k.starts_with('@'))
        && event.state_key() != Some(event.sender().as_str())
    {
        return false; // permission required to post in this room
    }

    true
}

/// Check if the given power levels event is authorized.
fn check_power_levels(
    room_version: &RoomVersion,
    power_event: impl Event,
    previous_power_event: Option<impl Event>,
    user_level: Int,
) -> Option<bool> {
    match power_event.state_key() {
        Some("") => {}
        Some(key) => {
            error!(state_key = key, "m.room.power_levels event has non-empty state key");
            return None;
        }
        None => {
            error!("check_power_levels requires an m.room.power_levels *state* event argument");
            return None;
        }
    }

    // Since v10, if any of the properties users_default, events_default, state_default, ban,
    // redact, kick, or invite in content are present and not an integer, reject.
    //
    // Since v10, if either of the properties events or notifications in content are present and not
    // a dictionary with values that are integers, reject.
    //
    // v1-v9, If the users property in content is not an object with keys that are valid user IDs
    // with values that are integers (or a string that is an integer), reject.
    // Since v10, if the users property in content is not an object with keys that are valid user
    // IDs with values that are integers, reject.
    let user_content: RoomPowerLevelsEventContent =
        deserialize_power_levels(power_event.content().get(), room_version)?;

    // Validation of users is done in Ruma, synapse for loops validating user_ids and integers here
    debug!("validation of power event finished");

    let current_state = match previous_power_event {
        Some(current_state) => current_state,
        // If there is no previous m.room.power_levels event in the room, allow
        None => return Some(true),
    };

    let current_content: RoomPowerLevelsEventContent =
        deserialize_power_levels(current_state.content().get(), room_version)?;

    // Since v1, for the properties users_default, events_default, state_default, ban, redact, kick,
    // invite check if they were added, changed or removed. For each found alteration:
    //
    // FIXME: this only performs the check if both the current value and the new value are present.
    let levels =
        ["users_default", "events_default", "state_default", "ban", "redact", "kick", "invite"];
    let current_state = serde_json::to_value(&current_content).unwrap();
    let new_state = serde_json::to_value(&user_content).unwrap();
    for lvl_name in &levels {
        if let Some((current_value, new_value)) =
            get_deserialize_levels(&current_state, &new_state, lvl_name)
        {
            // Since v1, if the current value is higher than the sender’s current power level,
            // reject.
            let current_value_too_big = current_value > user_level;
            // Since v1, If the new value is higher than the sender’s current power level, reject.
            let new_value_too_big = new_value > user_level;

            if current_value_too_big || new_value_too_big {
                warn!("cannot add ops > than own");
                return Some(false);
            }
        }
    }

    let mut event_levels_to_check = BTreeSet::new();
    let old_list = &current_content.events;
    let new_list = &user_content.events;
    for ev_id in old_list.keys().chain(new_list.keys()) {
        event_levels_to_check.insert(ev_id);
    }

    trace!(set = ?event_levels_to_check, "event levels to check");

    for ev_type in event_levels_to_check {
        let current_value = current_content.events.get(ev_type);
        let new_value = user_content.events.get(ev_type);
        // FIXME: testing for equality should be enough.
        if current_value.is_some() && new_value.is_some() && current_value == new_value {
            continue;
        }

        // Since v1, for each entry being changed in, or removed from, the events property:
        // - Since v1, if the current value is higher than the sender's current power level, reject.
        let current_value_too_big = current_value > Some(&user_level);
        // Since v1, for each entry being added to, or changed in, the events property:
        // - Since v1, if the new value is higher than the sender's current power level, reject.
        let new_value_too_big = new_value > Some(&user_level);
        if current_value_too_big || new_value_too_big {
            warn!("m.room.power_level failed to add ops > than own");
            return Some(false); // cannot add ops greater than own
        }
    }

    if room_version.limit_notifications_power_levels {
        let current_value = current_content.notifications.room;
        let new_value = user_content.notifications.room;
        if current_value != new_value {
            // Since v6, for each entry being changed in, or removed from, the notifications
            // property:
            // - Since v6, if the current value is higher than the sender's current power level,
            //   reject.
            let current_value_too_big = current_value > user_level;
            // Since v6, for each entry being added to, or changed in, the notifications property:
            // - Since v6, if the new value is higher than the sender's current power level, reject.
            let new_value_too_big = new_value > user_level;
            if current_value_too_big || new_value_too_big {
                warn!("m.room.power_level failed to add ops > than own");
                return Some(false); // cannot add ops greater than own
            }
        }
    }

    let mut user_levels_to_check = BTreeSet::new();
    let current_list = &current_content.users;
    let new_list = &user_content.users;
    for user in current_list.keys().chain(new_list.keys()) {
        let user: &UserId = user;
        user_levels_to_check.insert(user);
    }

    trace!(set = ?user_levels_to_check, "user levels to check");

    for user in user_levels_to_check {
        let current_value = current_content.users.get(user);
        let new_value = user_content.users.get(user);
        // FIXME: testing for equality should be enough.
        if current_value.is_some() && new_value.is_some() && current_value == new_value {
            continue;
        }

        // Since v1, for each entry being changed in, or removed from, the users property, other
        // than the sender’s own entry:
        // - Since v1, if the current value is greater than or equal to the sender’s current power
        //   level, reject.
        if user != power_event.sender() && current_value == Some(&user_level) {
            warn!("m.room.power_level cannot remove ops == to own");
            return Some(false); // cannot remove ops level == to own
        }

        let current_value_too_big = current_value > Some(&user_level);
        // Since v1, for each entry being added to, or changed in, the users property:
        // - Since v1, if the new value is greater than the sender’s current power level, reject.
        let new_value_too_big = new_value > Some(&user_level);
        if current_value_too_big || new_value_too_big {
            warn!("m.room.power_level failed to add ops > than own");
            return Some(false); // cannot add ops greater than own
        }
    }

    // Otherwise, allow.
    Some(true)
}

fn get_deserialize_levels(
    old: &serde_json::Value,
    new: &serde_json::Value,
    name: &str,
) -> Option<(Int, Int)> {
    Some((
        serde_json::from_value(old.get(name)?.clone()).ok()?,
        serde_json::from_value(new.get(name)?.clone()).ok()?,
    ))
}

/// Does the event redacting come from a user with enough power to redact the given event.
fn check_redaction(
    _room_version: &RoomVersion,
    redaction_event: impl Event,
    user_level: Int,
    redact_level: Int,
) -> Result<bool> {
    // v1-v2, if the sender’s power level is greater than or equal to the redact level, allow.
    if user_level >= redact_level {
        info!("redaction allowed via power levels");
        return Ok(true);
    }

    // v1-v2, if the domain of the event_id of the event being redacted is the same as the
    // domain of the event_id of the m.room.redaction, allow.
    if redaction_event.event_id().borrow().server_name()
        == redaction_event.redacts().as_ref().and_then(|&id| id.borrow().server_name())
    {
        info!("redaction event allowed via room version 1 rules");
        return Ok(true);
    }

    // Otherwise, reject.
    Ok(false)
}

/// Helper function to fetch the power level needed to send an event of type
/// `e_type` based on the rooms "m.room.power_level" event.
fn get_send_level(
    e_type: &TimelineEventType,
    state_key: Option<&str>,
    power_lvl: Option<impl Event>,
) -> Int {
    power_lvl
        .and_then(|ple| {
            from_json_str::<RoomPowerLevelsEventContent>(ple.content().get())
                .map(|content| {
                    content.events.get(e_type).copied().unwrap_or_else(|| {
                        if state_key.is_some() {
                            content.state_default
                        } else {
                            content.events_default
                        }
                    })
                })
                .ok()
        })
        .unwrap_or_else(|| if state_key.is_some() { int!(50) } else { int!(0) })
}

fn verify_third_party_invite(
    target_user: Option<&UserId>,
    sender: &UserId,
    tp_id: &ThirdPartyInvite,
    current_third_party_invite: Option<impl Event>,
) -> bool {
    // Checked during deserialization:
    // Since v1, if content.third_party_invite does not have a signed property, reject.
    // Since v1, if signed does not have mxid and token properties, reject.

    // Since v1, if mxid does not match state_key, reject.
    if target_user != Some(&tp_id.signed.mxid) {
        return false;
    }

    // Since v1, if there is no m.room.third_party_invite event in the current room state with
    // state_key matching token, reject.
    let current_tpid = match current_third_party_invite {
        Some(id) => id,
        None => return false,
    };

    // Since v1, if sender does not match sender of the m.room.third_party_invite, reject.
    if current_tpid.state_key() != Some(&tp_id.signed.token) {
        return false;
    }

    if sender != current_tpid.sender() {
        return false;
    }

    // Since v1, if any signature in signed matches any public key in the m.room.third_party_invite
    // event, allow. The public keys are in content of m.room.third_party_invite as:
    //
    // - A single public key in the public_key property.
    // - A list of public keys in the public_keys property.
    //
    // Otherwise, reject.
    //
    // FIXME: This does not check if the signature matches a public key, it checks if the token
    // matches a public key?
    let tpid_ev =
        match from_json_str::<RoomThirdPartyInviteEventContent>(current_tpid.content().get()) {
            Ok(ev) => ev,
            Err(_) => return false,
        };

    let decoded_invite_token = match Base64::parse(&tp_id.signed.token) {
        Ok(tok) => tok,
        // FIXME: Log a warning?
        Err(_) => return false,
    };

    // A list of public keys in the public_keys field
    for key in tpid_ev.public_keys.unwrap_or_default() {
        if key.public_key == decoded_invite_token {
            return true;
        }
    }

    // A single public key in the public_key field
    tpid_ev.public_key == decoded_invite_token
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use ruma_events::{
        room::{
            join_rules::{
                AllowRule, JoinRule, Restricted, RoomJoinRulesEventContent, RoomMembership,
            },
            member::{MembershipState, RoomMemberEventContent},
        },
        StateEventType, TimelineEventType,
    };
    use serde_json::value::to_raw_value as to_raw_json_value;

    use crate::{
        event_auth::valid_membership_change,
        test_utils::{
            alice, charlie, ella, event_id, member_content_ban, member_content_join, room_id,
            to_pdu_event, PduEvent, INITIAL_EVENTS, INITIAL_EVENTS_CREATE_ROOM,
        },
        Event, EventTypeExt, RoomVersion, StateMap,
    };

    #[test]
    fn test_ban_pass() {
        let _ =
            tracing::subscriber::set_default(tracing_subscriber::fmt().with_test_writer().finish());
        let events = INITIAL_EVENTS();

        let auth_events = events
            .values()
            .map(|ev| (ev.event_type().with_state_key(ev.state_key().unwrap()), Arc::clone(ev)))
            .collect::<StateMap<_>>();

        let requester = to_pdu_event(
            "HELLO",
            alice(),
            TimelineEventType::RoomMember,
            Some(charlie().as_str()),
            member_content_ban(),
            &[],
            &["IMC"],
        );

        let fetch_state = |ty, key| auth_events.get(&(ty, key)).cloned();
        let target_user = charlie();
        let sender = alice();

        assert!(valid_membership_change(
            &RoomVersion::V6,
            target_user,
            fetch_state(StateEventType::RoomMember, target_user.to_string()),
            sender,
            fetch_state(StateEventType::RoomMember, sender.to_string()),
            &requester,
            None::<PduEvent>,
            fetch_state(StateEventType::RoomPowerLevels, "".to_owned()),
            fetch_state(StateEventType::RoomJoinRules, "".to_owned()),
            None,
            &MembershipState::Leave,
            fetch_state(StateEventType::RoomCreate, "".to_owned()).unwrap(),
        )
        .unwrap());
    }

    #[test]
    fn test_join_non_creator() {
        let _ =
            tracing::subscriber::set_default(tracing_subscriber::fmt().with_test_writer().finish());
        let events = INITIAL_EVENTS_CREATE_ROOM();

        let auth_events = events
            .values()
            .map(|ev| (ev.event_type().with_state_key(ev.state_key().unwrap()), Arc::clone(ev)))
            .collect::<StateMap<_>>();

        let requester = to_pdu_event(
            "HELLO",
            charlie(),
            TimelineEventType::RoomMember,
            Some(charlie().as_str()),
            member_content_join(),
            &["CREATE"],
            &["CREATE"],
        );

        let fetch_state = |ty, key| auth_events.get(&(ty, key)).cloned();
        let target_user = charlie();
        let sender = charlie();

        assert!(!valid_membership_change(
            &RoomVersion::V6,
            target_user,
            fetch_state(StateEventType::RoomMember, target_user.to_string()),
            sender,
            fetch_state(StateEventType::RoomMember, sender.to_string()),
            &requester,
            None::<PduEvent>,
            fetch_state(StateEventType::RoomPowerLevels, "".to_owned()),
            fetch_state(StateEventType::RoomJoinRules, "".to_owned()),
            None,
            &MembershipState::Leave,
            fetch_state(StateEventType::RoomCreate, "".to_owned()).unwrap(),
        )
        .unwrap());
    }

    #[test]
    fn test_join_creator() {
        let _ =
            tracing::subscriber::set_default(tracing_subscriber::fmt().with_test_writer().finish());
        let events = INITIAL_EVENTS_CREATE_ROOM();

        let auth_events = events
            .values()
            .map(|ev| (ev.event_type().with_state_key(ev.state_key().unwrap()), Arc::clone(ev)))
            .collect::<StateMap<_>>();

        let requester = to_pdu_event(
            "HELLO",
            alice(),
            TimelineEventType::RoomMember,
            Some(alice().as_str()),
            member_content_join(),
            &["CREATE"],
            &["CREATE"],
        );

        let fetch_state = |ty, key| auth_events.get(&(ty, key)).cloned();
        let target_user = alice();
        let sender = alice();

        assert!(valid_membership_change(
            &RoomVersion::V6,
            target_user,
            fetch_state(StateEventType::RoomMember, target_user.to_string()),
            sender,
            fetch_state(StateEventType::RoomMember, sender.to_string()),
            &requester,
            None::<PduEvent>,
            fetch_state(StateEventType::RoomPowerLevels, "".to_owned()),
            fetch_state(StateEventType::RoomJoinRules, "".to_owned()),
            None,
            &MembershipState::Leave,
            fetch_state(StateEventType::RoomCreate, "".to_owned()).unwrap(),
        )
        .unwrap());
    }

    #[test]
    fn test_ban_fail() {
        let _ =
            tracing::subscriber::set_default(tracing_subscriber::fmt().with_test_writer().finish());
        let events = INITIAL_EVENTS();

        let auth_events = events
            .values()
            .map(|ev| (ev.event_type().with_state_key(ev.state_key().unwrap()), Arc::clone(ev)))
            .collect::<StateMap<_>>();

        let requester = to_pdu_event(
            "HELLO",
            charlie(),
            TimelineEventType::RoomMember,
            Some(alice().as_str()),
            member_content_ban(),
            &[],
            &["IMC"],
        );

        let fetch_state = |ty, key| auth_events.get(&(ty, key)).cloned();
        let target_user = alice();
        let sender = charlie();

        assert!(!valid_membership_change(
            &RoomVersion::V6,
            target_user,
            fetch_state(StateEventType::RoomMember, target_user.to_string()),
            sender,
            fetch_state(StateEventType::RoomMember, sender.to_string()),
            &requester,
            None::<PduEvent>,
            fetch_state(StateEventType::RoomPowerLevels, "".to_owned()),
            fetch_state(StateEventType::RoomJoinRules, "".to_owned()),
            None,
            &MembershipState::Leave,
            fetch_state(StateEventType::RoomCreate, "".to_owned()).unwrap(),
        )
        .unwrap());
    }

    #[test]
    fn test_restricted_join_rule() {
        let _ =
            tracing::subscriber::set_default(tracing_subscriber::fmt().with_test_writer().finish());
        let mut events = INITIAL_EVENTS();
        *events.get_mut(&event_id("IJR")).unwrap() = to_pdu_event(
            "IJR",
            alice(),
            TimelineEventType::RoomJoinRules,
            Some(""),
            to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Restricted(
                Restricted::new(vec![AllowRule::RoomMembership(RoomMembership::new(
                    room_id().to_owned(),
                ))]),
            )))
            .unwrap(),
            &["CREATE", "IMA", "IPOWER"],
            &["IPOWER"],
        );

        let mut member = RoomMemberEventContent::new(MembershipState::Join);
        member.join_authorized_via_users_server = Some(alice().to_owned());

        let auth_events = events
            .values()
            .map(|ev| (ev.event_type().with_state_key(ev.state_key().unwrap()), Arc::clone(ev)))
            .collect::<StateMap<_>>();

        let requester = to_pdu_event(
            "HELLO",
            ella(),
            TimelineEventType::RoomMember,
            Some(ella().as_str()),
            to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Join)).unwrap(),
            &["CREATE", "IJR", "IPOWER", "new"],
            &["new"],
        );

        let fetch_state = |ty, key| auth_events.get(&(ty, key)).cloned();
        let target_user = ella();
        let sender = ella();

        assert!(valid_membership_change(
            &RoomVersion::V9,
            target_user,
            fetch_state(StateEventType::RoomMember, target_user.to_string()),
            sender,
            fetch_state(StateEventType::RoomMember, sender.to_string()),
            &requester,
            None::<PduEvent>,
            fetch_state(StateEventType::RoomPowerLevels, "".to_owned()),
            fetch_state(StateEventType::RoomJoinRules, "".to_owned()),
            Some(alice()),
            &MembershipState::Join,
            fetch_state(StateEventType::RoomCreate, "".to_owned()).unwrap(),
        )
        .unwrap());

        assert!(!valid_membership_change(
            &RoomVersion::V9,
            target_user,
            fetch_state(StateEventType::RoomMember, target_user.to_string()),
            sender,
            fetch_state(StateEventType::RoomMember, sender.to_string()),
            &requester,
            None::<PduEvent>,
            fetch_state(StateEventType::RoomPowerLevels, "".to_owned()),
            fetch_state(StateEventType::RoomJoinRules, "".to_owned()),
            Some(ella()),
            &MembershipState::Leave,
            fetch_state(StateEventType::RoomCreate, "".to_owned()).unwrap(),
        )
        .unwrap());
    }

    #[test]
    fn test_knock() {
        let _ =
            tracing::subscriber::set_default(tracing_subscriber::fmt().with_test_writer().finish());
        let mut events = INITIAL_EVENTS();
        *events.get_mut(&event_id("IJR")).unwrap() = to_pdu_event(
            "IJR",
            alice(),
            TimelineEventType::RoomJoinRules,
            Some(""),
            to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Knock)).unwrap(),
            &["CREATE", "IMA", "IPOWER"],
            &["IPOWER"],
        );

        let auth_events = events
            .values()
            .map(|ev| (ev.event_type().with_state_key(ev.state_key().unwrap()), Arc::clone(ev)))
            .collect::<StateMap<_>>();

        let requester = to_pdu_event(
            "HELLO",
            ella(),
            TimelineEventType::RoomMember,
            Some(ella().as_str()),
            to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Knock)).unwrap(),
            &[],
            &["IMC"],
        );

        let fetch_state = |ty, key| auth_events.get(&(ty, key)).cloned();
        let target_user = ella();
        let sender = ella();

        assert!(valid_membership_change(
            &RoomVersion::V7,
            target_user,
            fetch_state(StateEventType::RoomMember, target_user.to_string()),
            sender,
            fetch_state(StateEventType::RoomMember, sender.to_string()),
            &requester,
            None::<PduEvent>,
            fetch_state(StateEventType::RoomPowerLevels, "".to_owned()),
            fetch_state(StateEventType::RoomJoinRules, "".to_owned()),
            None,
            &MembershipState::Leave,
            fetch_state(StateEventType::RoomCreate, "".to_owned()).unwrap(),
        )
        .unwrap());
    }
}
