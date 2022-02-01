use std::{borrow::Borrow, collections::BTreeSet, convert::TryFrom};

use js_int::{int, Int};
use ruma_events::{
    room::{
        create::RoomCreateEventContent,
        join_rules::{JoinRule, RoomJoinRulesEventContent},
        member::{MembershipState, ThirdPartyInvite},
        power_levels::RoomPowerLevelsEventContent,
        third_party_invite::RoomThirdPartyInviteEventContent,
    },
    EventType,
};
use ruma_identifiers::{RoomVersionId, UserId};
use ruma_serde::{Base64, Raw};
use serde::{de::IgnoredAny, Deserialize};
use serde_json::{from_str as from_json_str, value::RawValue as RawJsonValue};
use tracing::{debug, error, info, warn};

use crate::{room_version::RoomVersion, Error, Event, PowerLevelsContentFields, Result};

// FIXME: field extracting could be bundled for `content`
#[derive(Deserialize)]
struct GetMembership {
    membership: MembershipState,
}

#[derive(Deserialize)]
struct RoomMemberContentFields {
    membership: Option<Raw<MembershipState>>,
    #[cfg(feature = "unstable-spec")]
    join_authorised_via_users_server: Option<Raw<Box<UserId>>>,
}

#[derive(Deserialize)]
struct PowerLevelsContentInvite {
    invite: Int,
}

/// For the given event `kind` what are the relevant auth events that are needed to authenticate
/// this `content`.
///
/// # Errors
///
/// This function will return an error if the supplied `content` is not a JSON object.
pub fn auth_types_for_event(
    kind: &EventType,
    sender: &UserId,
    state_key: Option<&str>,
    content: &RawJsonValue,
) -> serde_json::Result<Vec<(EventType, String)>> {
    if kind == &EventType::RoomCreate {
        return Ok(vec![]);
    }

    let mut auth_types = vec![
        (EventType::RoomPowerLevels, "".to_owned()),
        (EventType::RoomMember, sender.to_string()),
        (EventType::RoomCreate, "".to_owned()),
    ];

    if kind == &EventType::RoomMember {
        #[derive(Deserialize)]
        struct RoomMemberContentFields {
            membership: Option<Raw<MembershipState>>,
            third_party_invite: Option<Raw<ThirdPartyInvite>>,
        }

        if let Some(state_key) = state_key {
            let content: RoomMemberContentFields = from_json_str(content.get())?;

            if let Some(Ok(membership)) = content.membership.map(|m| m.deserialize()) {
                if [MembershipState::Join, MembershipState::Invite].contains(&membership) {
                    let key = (EventType::RoomJoinRules, "".to_owned());
                    if !auth_types.contains(&key) {
                        auth_types.push(key);
                    }
                }

                let key = (EventType::RoomMember, state_key.to_owned());
                if !auth_types.contains(&key) {
                    auth_types.push(key);
                }

                if membership == MembershipState::Invite {
                    if let Some(Ok(t_id)) = content.third_party_invite.map(|t| t.deserialize()) {
                        let key = (EventType::RoomThirdPartyInvite, t_id.signed.token);
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

/// Authenticate the incoming `event`.
///
/// The steps of authentication are:
///
/// * check that the event is being authenticated for the correct room
/// * then there are checks for specific event types
///
/// The `fetch_state` closure should gather state from a state snapshot. We need to know if the
/// event passes auth against some state not a recursive collection of auth_events fields.
pub fn auth_check<E: Event>(
    room_version: &RoomVersion,
    incoming_event: impl Event,
    prev_event: Option<impl Event>,
    current_third_party_invite: Option<impl Event>,
    fetch_state: impl Fn(&EventType, &str) -> Option<E>,
) -> Result<bool> {
    info!(
        "auth_check beginning for {} ({})",
        incoming_event.event_id(),
        incoming_event.event_type()
    );

    // [synapse] check that all the events are in the same room as `incoming_event`

    // [synapse] do_sig_check check the event has valid signatures for member events

    // TODO do_size_check is false when called by `iterative_auth_check`
    // do_size_check is also mostly accomplished by ruma with the exception of checking event_type,
    // state_key, and json are below a certain size (255 and 65_536 respectively)

    let sender = incoming_event.sender();

    // Implementation of https://matrix.org/docs/spec/rooms/v1#authorization-rules
    //
    // 1. If type is m.room.create:
    if *incoming_event.event_type() == EventType::RoomCreate {
        #[derive(Deserialize)]
        struct RoomCreateContentFields {
            room_version: Option<Raw<RoomVersionId>>,
            creator: Option<Raw<IgnoredAny>>,
        }

        info!("start m.room.create check");

        // If it has any previous events, reject
        if incoming_event.prev_events().next().is_some() {
            warn!("the room creation event had previous events");
            return Ok(false);
        }

        // If the domain of the room_id does not match the domain of the sender, reject
        if incoming_event.room_id().server_name() != sender.server_name() {
            warn!("creation events server does not match sender");
            return Ok(false); // creation events room id does not match senders
        }

        let content: RoomCreateContentFields = from_json_str(incoming_event.content().get())?;

        // If content.room_version is present and is not a recognized version, reject
        if content.room_version.map(|v| v.deserialize().is_err()).unwrap_or(false) {
            warn!("invalid room version found in m.room.create event");
            return Ok(false);
        }

        // If content has no creator field, reject
        if content.creator.is_none() {
            warn!("no creator field found in m.room.create content");
            return Ok(false);
        }

        info!("m.room.create event was allowed");
        return Ok(true);
    }

    /*
    // 2. Reject if auth_events
    // a. auth_events cannot have duplicate keys since it's a BTree
    // b. All entries are valid auth events according to spec
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

    let room_create_event = fetch_state(&EventType::RoomCreate, "");

    // 3. If event does not have m.room.create in auth_events reject
    if room_create_event.is_none() {
        warn!("no m.room.create event in auth chain");

        return Ok(false);
    }

    // [synapse] checks for federation here

    // 4. If type is m.room.aliases
    if *incoming_event.event_type() == EventType::RoomAliases
        && room_version.special_case_aliases_auth
    {
        info!("starting m.room.aliases check");

        // If sender's domain doesn't matches state_key, reject
        if incoming_event.state_key() != Some(sender.server_name().as_str()) {
            warn!("state_key does not match sender");
            return Ok(false);
        }

        info!("m.room.aliases event was allowed");
        return Ok(true);
    }

    let power_levels_event = fetch_state(&EventType::RoomPowerLevels, "");
    let sender_member_event = fetch_state(&EventType::RoomMember, sender.as_str());

    if *incoming_event.event_type() == EventType::RoomMember {
        info!("starting m.room.member check");
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
            <&UserId>::try_from(state_key).map_err(|e| Error::InvalidPdu(format!("{}", e)))?;

        #[cfg(feature = "unstable-spec")]
        let join_authed_user =
            content.join_authorised_via_users_server.as_ref().and_then(|u| u.deserialize().ok());
        #[cfg(feature = "unstable-spec")]
        let join_authed_user_membership = if let Some(auth_user) = &join_authed_user {
            fetch_state(&EventType::RoomMember, auth_user.as_str())
                .and_then(|mem| from_json_str::<GetMembership>(mem.content().get()).ok())
                .map(|mem| mem.membership)
        } else {
            None
        };
        if !valid_membership_change(
            room_version,
            target_user,
            fetch_state(&EventType::RoomMember, target_user.as_str()).as_ref(),
            sender,
            sender_member_event.as_ref(),
            incoming_event.content(),
            prev_event,
            current_third_party_invite,
            power_levels_event.as_ref(),
            fetch_state(&EventType::RoomJoinRules, "").as_ref(),
            #[cfg(feature = "unstable-spec")]
            join_authed_user.as_deref(),
            #[cfg(feature = "unstable-spec")]
            join_authed_user_membership,
        )? {
            return Ok(false);
        }

        info!("m.room.member event was allowed");
        return Ok(true);
    }

    // If the sender's current membership state is not join, reject
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
        if let Ok(content) = from_json_str::<PowerLevelsContentFields>(pl.content().get()) {
            if let Some(level) = content.users.get(sender) {
                *level
            } else {
                content.users_default
            }
        } else {
            int!(0) // TODO if this fails DB error?
        }
    } else {
        // If no power level event found the creator gets 100 everyone else gets 0
        room_create_event
            .and_then(|create| from_json_str::<RoomCreateEventContent>(create.content().get()).ok())
            .and_then(|create| (create.creator == *sender).then(|| int!(100)))
            .unwrap_or_default()
    };

    // Allow if and only if sender's current power level is greater than
    // or equal to the invite level
    if *incoming_event.event_type() == EventType::RoomThirdPartyInvite {
        let invite_level = match &power_levels_event {
            Some(power_levels) => {
                from_json_str::<PowerLevelsContentInvite>(power_levels.content().get())?.invite
            }
            None => int!(50),
        };

        if sender_power_level < invite_level {
            warn!("sender's cannot send invites in this room");
            return Ok(false);
        }
    }

    // If the event type's required power level is greater than the sender's power level, reject
    // If the event has a state_key that starts with an @ and does not match the sender, reject.
    if !can_send_event(&incoming_event, power_levels_event.as_ref(), sender_power_level) {
        warn!("user cannot send event");
        return Ok(false);
    }

    if *incoming_event.event_type() == EventType::RoomPowerLevels {
        info!("starting m.room.power_levels check");

        if let Some(required_pwr_lvl) = check_power_levels(
            room_version,
            &incoming_event,
            power_levels_event.as_ref(),
            sender_power_level,
        ) {
            if !required_pwr_lvl {
                warn!("power level was not allowed");
                return Ok(false);
            }
        } else {
            warn!("power level was not allowed");
            return Ok(false);
        }
        info!("power levels event allowed");
    }

    // Room version 3: Redaction events are always accepted (provided the event is allowed by
    // `events` and `events_default` in the power levels). However, servers should not apply or
    // send redaction's to clients until both the redaction event and original event have been
    // seen, and are valid. Servers should only apply redaction's to events where the sender's
    // domains match, or the sender of the redaction has the appropriate permissions per the
    // power levels.

    if room_version.extra_redaction_checks
        && *incoming_event.event_type() == EventType::RoomRedaction
    {
        #[derive(Deserialize)]
        struct PowerLevelsContentRedact {
            redact: Int,
        }

        let redact_level = power_levels_event
            .and_then(|pl| from_json_str::<PowerLevelsContentRedact>(pl.content().get()).ok())
            .map(|c| c.redact)
            .unwrap_or_else(|| int!(50));

        if !check_redaction(room_version, incoming_event, sender_power_level, redact_level)? {
            return Ok(false);
        }
    }

    info!("allowing event passed all checks");
    Ok(true)
}

// TODO deserializing the member, power, join_rules event contents is done in conduit
// just before this is called. Could they be passed in?
/// Does the user who sent this member event have required power levels to do so.
///
/// * `user` - Information about the membership event and user making the request.
/// * `prev_event` - The event that occurred immediately before the `user` event or None.
/// * `auth_events` - The set of auth events that relate to a membership event.
///
/// This is generated by calling `auth_types_for_event` with the membership event and the current
/// State.
#[allow(clippy::too_many_arguments)]
fn valid_membership_change(
    room_version: &RoomVersion,
    target_user: &UserId,
    target_user_membership_event: Option<impl Event>,
    sender: &UserId,
    sender_membership_event: Option<impl Event>,
    content: &RawJsonValue,
    prev_event: Option<impl Event>,
    current_third_party_invite: Option<impl Event>,
    power_levels_event: Option<impl Event>,
    join_rules_event: Option<impl Event>,
    #[cfg(feature = "unstable-spec")] authed_user_id: Option<&UserId>,
    #[cfg(feature = "unstable-spec")] auth_user_membership: Option<MembershipState>,
) -> Result<bool> {
    #[derive(Deserialize)]
    struct GetThirdPartyInvite {
        third_party_invite: Option<Raw<ThirdPartyInvite>>,
    }

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

    let sender_power = power_levels
        .users
        .get(sender)
        .or_else(|| sender_is_joined.then(|| &power_levels.users_default));

    let target_power = power_levels.users.get(target_user).or_else(|| {
        (target_membership == MembershipState::Join).then(|| &power_levels.users_default)
    });

    let mut join_rules = JoinRule::Invite;
    if let Some(jr) = &join_rules_event {
        join_rules = from_json_str::<RoomJoinRulesEventContent>(jr.content().get())?.join_rule;
    }

    if let Some(prev) = prev_event {
        if *prev.event_type() == EventType::RoomCreate && prev.prev_events().next().is_none() {
            return Ok(true);
        }
    }

    let power_levels_event_id = power_levels_event.as_ref().map(|e| e.event_id());
    let sender_membership_event_id = sender_membership_event.as_ref().map(|e| e.event_id());
    let target_user_membership_event_id =
        target_user_membership_event.as_ref().map(|e| e.event_id());

    #[cfg(not(feature = "unstable-spec"))]
    let restricted = false;

    #[cfg(not(feature = "unstable-spec"))]
    let allow_based_on_membership = false;

    #[cfg(not(feature = "unstable-spec"))]
    let restricted_join_rules_auth = false;
    // FIXME: `JoinRule::Restricted(_)` can contain conditions that allow a user to join if
    // they are met. So far the spec talks about roomId based auth inheritance, the problem with
    // this is that ruma-state-res can only request events from one room at a time :(
    #[cfg(feature = "unstable-spec")]
    let restricted = matches!(join_rules, JoinRule::Restricted(_));

    #[cfg(feature = "unstable-spec")]
    let allow_based_on_membership =
        matches!(target_user_current_membership, MembershipState::Invite | MembershipState::Join)
            || authed_user_id.is_none();

    #[cfg(feature = "unstable-spec")]
    let restricted_join_rules_auth = if let Some(authed_user_id) = authed_user_id {
        // Is the authorised user allowed to invite users into this rooom
        let (auth_user_pl, invite_level) = if let Some(pl) = &power_levels_event {
            let invite = match from_json_str::<PowerLevelsContentInvite>(pl.content().get()) {
                Ok(power_levels) => power_levels.invite,
                _ => int!(50),
            };

            if let Ok(content) = from_json_str::<PowerLevelsContentFields>(pl.content().get()) {
                let user_pl = if let Some(level) = content.users.get(authed_user_id) {
                    *level
                } else {
                    content.users_default
                };

                (user_pl, invite)
            } else {
                (int!(0), invite)
            }
        } else {
            (int!(0), int!(0))
        };
        (auth_user_membership == Some(MembershipState::Join)) && (auth_user_pl >= invite_level)
    } else {
        // If the `join_authorised_via_users_server` was empty we treat the target user as invited
        true
    };

    Ok(match target_membership {
        MembershipState::Join => {
            if sender != target_user {
                warn!("Can't make other user join");
                false
            } else if let MembershipState::Ban = target_user_current_membership {
                warn!(?target_user_membership_event_id, "Banned user can't join");
                false
            } else {
                let allow = join_rules == JoinRule::Invite
                    && (target_user_current_membership == MembershipState::Join
                        || target_user_current_membership == MembershipState::Invite)
                    || join_rules == JoinRule::Public
                    || room_version.restricted_join_rules
                        // 0. room version of 8 ^ and join rule of restricted
                        && restricted
                        // 1. The user's previous membership was invite or join
                        && allow_based_on_membership
                        // 2. The join event has a valid signature from a homeserver whose
                        // users have the power to issue invites or the field was `None`.
                        && restricted_join_rules_auth;

                if !allow {
                    warn!(
                        join_rules_event_id = ?join_rules_event.as_ref().map(|e| e.event_id()),
                        ?target_user_membership_event_id,
                        "Can't join if join rules is not public and user is not invited / joined",
                    );
                }
                allow
            }
        }
        MembershipState::Invite => {
            // If content has third_party_invite key
            if let Some(tp_id) = third_party_invite.and_then(|i| i.deserialize().ok()) {
                if target_user_current_membership == MembershipState::Ban {
                    warn!(?target_user_membership_event_id, "Can't invite banned user");
                    false
                } else {
                    let allow = verify_third_party_invite(
                        Some(target_user),
                        sender,
                        &tp_id,
                        current_third_party_invite,
                    );
                    if !allow {
                        warn!("Third party invite invalid");
                    }
                    allow
                }
            } else if !sender_is_joined
                || target_user_current_membership == MembershipState::Join
                || target_user_current_membership == MembershipState::Ban
            {
                warn!(
                    ?target_user_membership_event_id,
                    ?sender_membership_event_id,
                    "Can't invite user if sender not joined or the user is currently joined or \
                     banned",
                );
                false
            } else {
                let allow = sender_power.filter(|&p| p >= &power_levels.invite).is_some();
                if !allow {
                    warn!(
                        ?target_user_membership_event_id,
                        ?power_levels_event_id,
                        "User does not have enough power to invite",
                    );
                }
                allow
            }
        }
        MembershipState::Leave => {
            if sender == target_user {
                let allow = target_user_current_membership == MembershipState::Join
                    || target_user_current_membership == MembershipState::Invite;
                if !allow {
                    warn!(?target_user_membership_event_id, "Can't leave if not invited or joined");
                }
                allow
            } else if !sender_is_joined
                || target_user_current_membership == MembershipState::Ban
                    && sender_power.filter(|&p| p < &power_levels.ban).is_some()
            {
                warn!(
                    ?target_user_membership_event_id,
                    ?sender_membership_event_id,
                    "Can't kick if sender not joined or user is already banned",
                );
                false
            } else {
                let allow = sender_power.filter(|&p| p >= &power_levels.kick).is_some()
                    && target_power < sender_power;
                if !allow {
                    warn!(
                        ?target_user_membership_event_id,
                        ?power_levels_event_id,
                        "User does not have enough power to kick",
                    );
                }
                allow
            }
        }
        MembershipState::Ban => {
            if !sender_is_joined {
                warn!(?sender_membership_event_id, "Can't ban user if sender is not joined");
                false
            } else {
                let allow = sender_power.filter(|&p| p >= &power_levels.ban).is_some()
                    && target_power < sender_power;
                if !allow {
                    warn!(
                        ?target_user_membership_event_id,
                        ?power_levels_event_id,
                        "User does not have enough power to ban",
                    );
                }
                allow
            }
        }
        MembershipState::Knock if room_version.allow_knocking => {
            // 1. If the `join_rule` is anything other than `knock`, reject.
            if join_rules != JoinRule::Knock {
                warn!("Join rule is not set to knock, knocking is not allowed");
                false
            } else {
                // 2. If `sender` does not match `state_key`, reject.
                // 3. If the `sender`'s current membership is not `ban`, `invite`, or `join`, allow.
                // 4. Otherwise, reject.
                if sender != target_user {
                    warn!(
                        ?sender,
                        ?target_user,
                        "Can't make another user join, sender did not match target"
                    );
                    false
                } else if matches!(
                    sender_membership,
                    MembershipState::Ban | MembershipState::Invite | MembershipState::Join
                ) {
                    warn!(
                        ?target_user_membership_event_id,
                        "Membership state of ban, invite, or join are invalid",
                    );
                    false
                } else {
                    true
                }
            }
        }
        _ => {
            warn!("Unknown membership transition");
            false
        }
    })
}

/// Is the user allowed to send a specific event based on the rooms power levels.
///
/// Does the event have the correct userId as its state_key if it's not the "" state_key.
fn can_send_event(event: impl Event, ple: Option<impl Event>, user_level: Int) -> bool {
    let event_type_power_level = get_send_level(event.event_type(), event.state_key(), ple);

    debug!("{} ev_type {} usr {}", event.event_id(), event_type_power_level, user_level);

    if user_level < event_type_power_level {
        return false;
    }

    if event.state_key().map_or(false, |k| k.starts_with('@'))
        && event.state_key() != Some(event.sender().as_str())
    {
        return false; // permission required to post in this room
    }

    true
}

/// Confirm that the event sender has the required power levels.
fn check_power_levels(
    room_version: &RoomVersion,
    power_event: impl Event,
    previous_power_event: Option<impl Event>,
    user_level: Int,
) -> Option<bool> {
    match power_event.state_key() {
        Some("") => {}
        Some(key) => {
            error!("m.room.power_levels event has non-empty state key: {}", key);
            return None;
        }
        None => {
            error!("check_power_levels requires an m.room.power_levels *state* event argument");
            return None;
        }
    }

    let current_state = match previous_power_event {
        Some(current_state) => current_state,
        // If there is no previous m.room.power_levels event in the room, allow
        None => return Some(true),
    };

    // If users key in content is not a dictionary with keys that are valid user IDs
    // with values that are integers (or a string that is an integer), reject.
    let user_content =
        from_json_str::<RoomPowerLevelsEventContent>(power_event.content().get()).unwrap();

    let current_content =
        from_json_str::<RoomPowerLevelsEventContent>(current_state.content().get()).unwrap();

    // Validation of users is done in Ruma, synapse for loops validating user_ids and integers here
    info!("validation of power event finished");

    let mut user_levels_to_check = BTreeSet::new();
    let old_list = &current_content.users;
    let user_list = &user_content.users;
    for user in old_list.keys().chain(user_list.keys()) {
        let user: &UserId = user;
        user_levels_to_check.insert(user);
    }

    debug!("users to check {:?}", user_levels_to_check);

    let mut event_levels_to_check = BTreeSet::new();
    let old_list = &current_content.events;
    let new_list = &user_content.events;
    for ev_id in old_list.keys().chain(new_list.keys()) {
        let ev_id: &EventType = ev_id;
        event_levels_to_check.insert(ev_id);
    }

    debug!("events to check {:?}", event_levels_to_check);

    let old_state = &current_content;
    let new_state = &user_content;

    // synapse does not have to split up these checks since we can't combine UserIds and
    // EventTypes we do 2 loops

    // UserId loop
    for user in user_levels_to_check {
        let old_level = old_state.users.get(user);
        let new_level = new_state.users.get(user);
        if old_level.is_some() && new_level.is_some() && old_level == new_level {
            continue;
        }

        // If the current value is equal to the sender's current power level, reject
        if user != power_event.sender() && old_level == Some(&user_level) {
            warn!("m.room.power_level cannot remove ops == to own");
            return Some(false); // cannot remove ops level == to own
        }

        // If the current value is higher than the sender's current power level, reject
        // If the new value is higher than the sender's current power level, reject
        let old_level_too_big = old_level > Some(&user_level);
        let new_level_too_big = new_level > Some(&user_level);
        if old_level_too_big || new_level_too_big {
            warn!("m.room.power_level failed to add ops > than own");
            return Some(false); // cannot add ops greater than own
        }
    }

    // EventType loop
    for ev_type in event_levels_to_check {
        let old_level = old_state.events.get(ev_type);
        let new_level = new_state.events.get(ev_type);
        if old_level.is_some() && new_level.is_some() && old_level == new_level {
            continue;
        }

        // If the current value is higher than the sender's current power level, reject
        // If the new value is higher than the sender's current power level, reject
        let old_level_too_big = old_level > Some(&user_level);
        let new_level_too_big = new_level > Some(&user_level);
        if old_level_too_big || new_level_too_big {
            warn!("m.room.power_level failed to add ops > than own");
            return Some(false); // cannot add ops greater than own
        }
    }

    // Notifications, currently there is only @room
    if room_version.limit_notifications_power_levels {
        let old_level = old_state.notifications.room;
        let new_level = new_state.notifications.room;
        if old_level != new_level {
            // If the current value is higher than the sender's current power level, reject
            // If the new value is higher than the sender's current power level, reject
            let old_level_too_big = old_level > user_level;
            let new_level_too_big = new_level > user_level;
            if old_level_too_big || new_level_too_big {
                warn!("m.room.power_level failed to add ops > than own");
                return Some(false); // cannot add ops greater than own
            }
        }
    }

    let levels =
        ["users_default", "events_default", "state_default", "ban", "redact", "kick", "invite"];
    let old_state = serde_json::to_value(old_state).unwrap();
    let new_state = serde_json::to_value(new_state).unwrap();
    for lvl_name in &levels {
        if let Some((old_lvl, new_lvl)) = get_deserialize_levels(&old_state, &new_state, lvl_name) {
            let old_level_too_big = old_lvl > user_level;
            let new_level_too_big = new_lvl > user_level;

            if old_level_too_big || new_level_too_big {
                warn!("cannot add ops > than own");
                return Some(false);
            }
        }
    }

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
    if user_level >= redact_level {
        info!("redaction allowed via power levels");
        return Ok(true);
    }

    // If the domain of the event_id of the event being redacted is the same as the
    // domain of the event_id of the m.room.redaction, allow
    if redaction_event.event_id().borrow().server_name()
        == redaction_event.redacts().as_ref().and_then(|&id| id.borrow().server_name())
    {
        info!("redaction event allowed via room version 1 rules");
        return Ok(true);
    }

    Ok(false)
}

/// Helper function to fetch the power level needed to send an event of type
/// `e_type` based on the rooms "m.room.power_level" event.
fn get_send_level(
    e_type: &EventType,
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
    // 1. Check for user being banned happens before this is called
    // checking for mxid and token keys is done by ruma when deserializing

    // The state key must match the invitee
    if target_user != Some(&tp_id.signed.mxid) {
        return false;
    }

    // If there is no m.room.third_party_invite event in the current room state with state_key
    // matching token, reject
    let current_tpid = match current_third_party_invite {
        Some(id) => id,
        None => return false,
    };

    if current_tpid.state_key() != Some(&tp_id.signed.token) {
        return false;
    }

    if sender != current_tpid.sender() {
        return false;
    }

    // If any signature in signed matches any public key in the m.room.third_party_invite event,
    // allow
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

    use crate::{
        event_auth::valid_membership_change,
        test_utils::{
            alice, charlie, ella, event_id, member_content_ban, to_pdu_event, StateEvent,
            INITIAL_EVENTS,
        },
        Event, RoomVersion, StateMap,
    };
    use ruma_events::room::{
        join_rules::{JoinRule, RoomJoinRulesEventContent},
        member::{MembershipState, RoomMemberEventContent},
    };
    use serde_json::value::to_raw_value as to_raw_json_value;
    #[cfg(feature = "unstable-spec")]
    use {
        crate::test_utils::{bob, room_id},
        ruma_events::room::join_rules::{AllowRule, Restricted, RoomMembership},
    };

    use ruma_events::EventType;

    #[test]
    fn test_ban_pass() {
        let _ =
            tracing::subscriber::set_default(tracing_subscriber::fmt().with_test_writer().finish());
        let events = INITIAL_EVENTS();

        let prev_event =
            events.values().find(|ev| ev.event_id.as_str().contains("IMC")).map(Arc::clone);

        let auth_events = events
            .values()
            .map(|ev| {
                ((ev.event_type().to_owned(), ev.state_key().unwrap().to_owned()), Arc::clone(ev))
            })
            .collect::<StateMap<_>>();

        let requester = to_pdu_event(
            "HELLO",
            alice(),
            EventType::RoomMember,
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
            &target_user,
            fetch_state(EventType::RoomMember, target_user.to_string()),
            &sender,
            fetch_state(EventType::RoomMember, sender.to_string()),
            requester.content(),
            prev_event,
            None::<StateEvent>,
            fetch_state(EventType::RoomPowerLevels, "".to_owned()),
            fetch_state(EventType::RoomJoinRules, "".to_owned()),
            #[cfg(feature = "unstable-spec")]
            None,
            #[cfg(feature = "unstable-spec")]
            None,
        )
        .unwrap());
    }

    #[test]
    fn test_ban_fail() {
        let _ =
            tracing::subscriber::set_default(tracing_subscriber::fmt().with_test_writer().finish());
        let events = INITIAL_EVENTS();

        let prev_event =
            events.values().find(|ev| ev.event_id.as_str().contains("IMC")).map(Arc::clone);

        let auth_events = events
            .values()
            .map(|ev| {
                ((ev.event_type().to_owned(), ev.state_key().unwrap().to_owned()), Arc::clone(ev))
            })
            .collect::<StateMap<_>>();

        let requester = to_pdu_event(
            "HELLO",
            charlie(),
            EventType::RoomMember,
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
            &target_user,
            fetch_state(EventType::RoomMember, target_user.to_string()),
            &sender,
            fetch_state(EventType::RoomMember, sender.to_string()),
            requester.content(),
            prev_event,
            None::<StateEvent>,
            fetch_state(EventType::RoomPowerLevels, "".to_owned()),
            fetch_state(EventType::RoomJoinRules, "".to_owned()),
            #[cfg(feature = "unstable-spec")]
            None,
            #[cfg(feature = "unstable-spec")]
            None,
        )
        .unwrap());
    }

    #[cfg(feature = "unstable-spec")]
    #[test]
    fn test_restricted_join_rule() {
        let _ =
            tracing::subscriber::set_default(tracing_subscriber::fmt().with_test_writer().finish());
        let mut events = INITIAL_EVENTS();
        *events.get_mut(&event_id("IJR")).unwrap() = to_pdu_event(
            "IJR",
            alice(),
            EventType::RoomJoinRules,
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

        let mut member = RoomMemberEventContent::new(MembershipState::Invite);
        member.join_authorized_via_users_server = Some(alice());
        events.insert(
            event_id("new"),
            to_pdu_event(
                "new",
                bob(),
                EventType::RoomMember,
                Some(ella().as_str()),
                to_raw_json_value(&member).unwrap(),
                &["CREATE", "IJR", "IPOWER"],
                &["IMC"],
            ),
        );

        let prev_event =
            events.values().find(|ev| ev.event_id.as_str().contains("IMC")).map(Arc::clone);

        let auth_events = events
            .values()
            .map(|ev| {
                ((ev.event_type().to_owned(), ev.state_key().unwrap().to_owned()), Arc::clone(ev))
            })
            .collect::<StateMap<_>>();

        let requester = to_pdu_event(
            "HELLO",
            ella(),
            EventType::RoomMember,
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
            &target_user,
            fetch_state(EventType::RoomMember, target_user.to_string()),
            &sender,
            fetch_state(EventType::RoomMember, sender.to_string()),
            requester.content(),
            prev_event.clone(),
            None::<StateEvent>,
            fetch_state(EventType::RoomPowerLevels, "".to_owned()),
            fetch_state(EventType::RoomJoinRules, "".to_owned()),
            Some(&alice()),
            Some(MembershipState::Join),
        )
        .unwrap());

        assert!(!valid_membership_change(
            &RoomVersion::V9,
            &target_user,
            fetch_state(EventType::RoomMember, target_user.to_string()),
            &sender,
            fetch_state(EventType::RoomMember, sender.to_string()),
            requester.content(),
            prev_event,
            None::<StateEvent>,
            fetch_state(EventType::RoomPowerLevels, "".to_owned()),
            fetch_state(EventType::RoomJoinRules, "".to_owned()),
            Some(&ella()),
            Some(MembershipState::Join),
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
            EventType::RoomJoinRules,
            Some(""),
            to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Knock)).unwrap(),
            &["CREATE", "IMA", "IPOWER"],
            &["IPOWER"],
        );

        let prev_event =
            events.values().find(|ev| ev.event_id.as_str().contains("IMC")).map(Arc::clone);

        let auth_events = events
            .values()
            .map(|ev| {
                ((ev.event_type().to_owned(), ev.state_key().unwrap().to_owned()), Arc::clone(ev))
            })
            .collect::<StateMap<_>>();

        let requester = to_pdu_event(
            "HELLO",
            ella(),
            EventType::RoomMember,
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
            &target_user,
            fetch_state(EventType::RoomMember, target_user.to_string()),
            &sender,
            fetch_state(EventType::RoomMember, sender.to_string()),
            requester.content(),
            prev_event,
            None::<StateEvent>,
            fetch_state(EventType::RoomPowerLevels, "".to_owned()),
            fetch_state(EventType::RoomJoinRules, "".to_owned()),
            #[cfg(feature = "unstable-spec")]
            None,
            #[cfg(feature = "unstable-spec")]
            None,
        )
        .unwrap());
    }
}
