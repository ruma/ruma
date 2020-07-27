use std::convert::TryFrom;

use maplit::btreeset;
use ruma::{
    events::{
        room::{self, join_rules::JoinRule, member::MembershipState},
        EventType,
    },
    identifiers::{RoomVersionId, UserId},
};
use serde_json::json;

use crate::{room_version::RoomVersion, state_event::StateEvent, StateMap};

/// Represents the 3 event redaction outcomes.
pub enum RedactAllowed {
    /// The event is the users so redaction can take place.
    OwnEvent,
    /// The user can easily redact the event.
    CanRedact,
    /// The user does not have enough power to redact this event.
    No,
}

pub fn auth_types_for_event(event: &StateEvent) -> Vec<(EventType, String)> {
    if event.kind() == EventType::RoomCreate {
        return vec![];
    }

    let mut auth_types = vec![
        (EventType::RoomPowerLevels, "".to_string()),
        (EventType::RoomMember, event.sender().to_string()),
        (EventType::RoomCreate, "".to_string()),
    ];

    if event.kind() == EventType::RoomMember {
        if let Ok(content) = event.deserialize_content::<room::member::MemberEventContent>() {
            if [MembershipState::Join, MembershipState::Invite].contains(&content.membership) {
                let key = (EventType::RoomJoinRules, "".into());
                if !auth_types.contains(&key) {
                    auth_types.push(key)
                }
            }

            // TODO what when we don't find a state_key
            let key = (EventType::RoomMember, event.state_key().unwrap());
            if !auth_types.contains(&key) {
                auth_types.push(key)
            }

            if content.membership == MembershipState::Invite {
                if let Some(t_id) = content.third_party_invite {
                    let key = (EventType::RoomThirdPartyInvite, t_id.signed.token);
                    if !auth_types.contains(&key) {
                        auth_types.push(key)
                    }
                }
            }
        }
    }

    auth_types
}

pub fn auth_check(
    room_version: &RoomVersionId,
    event: &StateEvent,
    auth_events: StateMap<StateEvent>,
    do_sig_check: bool,
) -> Option<bool> {
    tracing::info!("auth_check beginning");

    // don't let power from other rooms be used
    for auth_event in auth_events.values() {
        if auth_event.room_id() != event.room_id() {
            tracing::info!("found auth event that did not match event's room_id");
            return Some(false);
        }
    }

    if do_sig_check {
        let sender_domain = event.sender().server_name();

        let is_invite_via_3pid = if event.kind() == EventType::RoomMember {
            event
                .deserialize_content::<room::member::MemberEventContent>()
                .map(|c| c.membership == MembershipState::Invite && c.third_party_invite.is_some())
                .unwrap_or_default()
        } else {
            false
        };

        if event.signatures().get(sender_domain).is_none() && !is_invite_via_3pid {
            tracing::info!("event not signed by sender's server");
            return Some(false);
        }
    }

    // TODO do_size_check is false when called by `iterative_auth_check`
    // do_size_check is also mostly accomplished by ruma with the exception of checking event_type,
    // state_key, and json are below a certain size (255 and 65536 respectivly)

    // Implementation of https://matrix.org/docs/spec/rooms/v1#authorization-rules
    //
    // 1. If type is m.room.create:
    if event.kind() == EventType::RoomCreate {
        tracing::info!("start m.room.create check");

        // domain of room_id must match domain of sender.
        if event.room_id().map(|id| id.server_name()) != Some(event.sender().server_name()) {
            tracing::info!("creation events server does not match sender");
            return Some(false); // creation events room id does not match senders
        }

        // if content.room_version is present and is not a valid version
        // TODO check this out (what event has this as content?)
        if serde_json::from_value::<RoomVersionId>(
            event
                .content()
                .get("room_version")
                .cloned()
                // synapse defaults to version 1
                .unwrap_or(serde_json::json!("1")),
        )
        .is_err()
        {
            return Some(false);
        }

        tracing::info!("m.room.create event was allowed");
        return Some(true);
    }

    // 3. If event does not have m.room.create in auth_events reject.
    if auth_events
        .get(&(EventType::RoomCreate, "".into()))
        .is_none()
    {
        tracing::warn!("no m.room.create event in auth chain");

        return Some(false);
    }

    // check for m.federate
    if event.room_id().map(|id| id.server_name()) != Some(event.sender().server_name()) {
        tracing::info!("checking federation");

        if !can_federate(&auth_events) {
            tracing::warn!("federation not allowed");

            return Some(false);
        }
    }

    // 4. if type is m.room.aliases
    if event.kind() == EventType::RoomAliases {
        tracing::info!("starting m.room.aliases check");
        // TODO && room_version "special case aliases auth" ??
        if event.state_key().is_none() {
            return Some(false); // must have state_key
        }
        if event.state_key().unwrap().is_empty() {
            return Some(false); // and be non-empty state_key (point to a user_id)
        }

        if event.state_key() != Some(event.sender().to_string()) {
            return Some(false);
        }

        tracing::info!("m.room.aliases event was allowed");
        return Some(true);
    }

    if event.kind() == EventType::RoomMember {
        tracing::info!("starting m.room.member check");

        if !is_membership_change_allowed(event, &auth_events)? {
            return Some(false);
        }

        tracing::info!("m.room.member event was allowed");
        return Some(true);
    }

    if let Some(in_room) = check_event_sender_in_room(event, &auth_events) {
        if !in_room {
            tracing::warn!("sender not in room");
            return Some(false);
        }
    } else {
        tracing::warn!("sender not in room");
        return Some(false);
    }

    // Special case to allow m.room.third_party_invite events where ever
    // a user is allowed to issue invites
    if event.kind() == EventType::RoomThirdPartyInvite {
        // TODO impl this
        unimplemented!("third party invite")
    }

    if !can_send_event(event, &auth_events)? {
        tracing::warn!("user cannot send event");
        return Some(false);
    }

    if event.kind() == EventType::RoomPowerLevels {
        tracing::info!("starting m.room.power_levels check");
        if let Some(required_pwr_lvl) = check_power_levels(room_version, event, &auth_events) {
            if !required_pwr_lvl {
                tracing::warn!("power level was not allowed");
                return Some(false);
            }
        } else {
            tracing::warn!("power level was not allowed");
            return Some(false);
        }
        tracing::info!("power levels event allowed");
    }

    if event.kind() == EventType::RoomRedaction {
        if let RedactAllowed::No = check_redaction(room_version, event, &auth_events)? {
            return Some(false);
        }
    }

    tracing::info!("allowing event passed all checks");
    Some(true)
}

// synapse has an `event: &StateEvent` param but it's never used
/// Can this room federate based on its m.room.create event.
fn can_federate(auth_events: &StateMap<StateEvent>) -> bool {
    let creation_event = auth_events.get(&(EventType::RoomCreate, "".into()));
    if let Some(ev) = creation_event {
        if let Some(fed) = ev.content().get("m.federate") {
            fed == "true"
        } else {
            false
        }
    } else {
        false
    }
}

/// Dose the user who sent this member event have required power levels to do so.
fn is_membership_change_allowed(
    event: &StateEvent,
    auth_events: &StateMap<StateEvent>,
) -> Option<bool> {
    let content = event
        .deserialize_content::<room::member::MemberEventContent>()
        .ok()
        .unwrap();
    let membership = content.membership;

    // check if this is the room creator joining
    if event.prev_event_ids().len() == 1 && membership == MembershipState::Join {
        if let Some(create) = auth_events.get(&(EventType::RoomCreate, "".into())) {
            if let Ok(create_ev) = create.deserialize_content::<room::create::CreateEventContent>()
            {
                if event.state_key() == Some(create_ev.creator.to_string()) {
                    tracing::debug!("m.room.member event allowed via m.room.create");
                    return Some(true);
                }
            }
        }
    }

    let target_user_id = UserId::try_from(event.state_key().unwrap()).ok().unwrap();
    // if the server_names are different and federation is NOT allowed
    if event.room_id().unwrap().server_name() != target_user_id.server_name()
        && !can_federate(auth_events)
    {
        tracing::info!("server cannot federate");
        return Some(false);
    }

    let key = (EventType::RoomMember, event.sender().to_string());
    let caller = auth_events.get(&key);

    let caller_in_room = caller.is_some() && check_membership(caller, MembershipState::Join);
    let caller_invited = caller.is_some() && check_membership(caller, MembershipState::Invite);

    let key = (EventType::RoomMember, target_user_id.to_string());
    let target = auth_events.get(&key);

    let target_in_room = target.is_some() && check_membership(target, MembershipState::Join);
    let target_banned = target.is_some() && check_membership(target, MembershipState::Ban);

    let key = (EventType::RoomJoinRules, "".to_string());
    let join_rules_event = auth_events.get(&key);

    let mut join_rule = JoinRule::Invite;
    if let Some(jr) = join_rules_event {
        join_rule = jr
            .deserialize_content::<room::join_rules::JoinRulesEventContent>()
            .ok()
            .unwrap() // TODO these are errors? and should be treated as a DB failure?
            .join_rule;
    }

    let user_level = get_user_power_level(event.sender(), auth_events);
    let target_level = get_user_power_level(&target_user_id, auth_events);

    // synapse has a not "what to do for default here   50"
    let ban_level = get_named_level(auth_events, "ban", 50);

    // TODO clean this up
    tracing::debug!(
        "_is_membership_change_allowed: {}",
        serde_json::to_string_pretty(&json!({
            "caller_in_room": caller_in_room,
            "caller_invited": caller_invited,
            "target_banned": target_banned,
            "target_in_room": target_in_room,
            "membership": membership,
            "join_rule": join_rule,
            "target_user_id": target_user_id,
            "event.user_id": event.sender(),
        }))
        .unwrap(),
    );

    if membership == MembershipState::Invite && content.third_party_invite.is_some() {
        // TODO this is unimpled
        if !verify_third_party_invite(event, auth_events) {
            tracing::info!(
                "{} was not invited to this room",
                event
                    .event_id()
                    .map(ToString::to_string)
                    .unwrap_or("Unknow".into())
            );
            return Some(false);
        }
        if target_banned {
            tracing::info!(
                "{} is banned",
                event
                    .event_id()
                    .map(ToString::to_string)
                    .unwrap_or("Unknow".into())
            );
            return Some(false);
        }
        tracing::info!("invite succeded");
        return Some(true);
    }

    if membership != MembershipState::Join {
        if caller_invited
            && membership == MembershipState::Leave
            && &target_user_id == event.sender()
        {
            tracing::info!("join event succeded");
            return Some(true);
        }

        if !caller_in_room {
            tracing::info!(
                "{} is not in this room {:?}",
                event.sender(),
                event.room_id()
            );
            return Some(false); // caller is not joined
        }
    }

    if membership == MembershipState::Invite {
        if target_banned {
            tracing::info!("target has been banned");
            return Some(false);
        } else if target_in_room {
            tracing::info!("already in room");
            return Some(false); // already in room
        } else {
            let invite_level = get_named_level(auth_events, "invite", 0);
            if user_level < invite_level {
                return Some(false);
            }
        }
    } else if membership == MembershipState::Join {
        if event.sender() != &target_user_id {
            tracing::info!("cannot force another user to join");
            return Some(false); // cannot force another user to join
        } else if target_banned {
            tracing::info!("cannot join when banned");
            return Some(false); // cannot joined when banned
        } else if join_rule == JoinRule::Public {
            tracing::info!("join rule public")
        // pass
        } else if join_rule == JoinRule::Invite {
            if !caller_in_room && !caller_invited {
                tracing::info!("user has not been invited to this room");
                return Some(false); // you are not invited to this room
            }
        } else {
            tracing::info!("the join rule is Private or yet to be spec'ed by Matrix");
            // synapse has 2 TODO's may_join list and private rooms

            // the join_rule is Private or Knock which means it is not yet spec'ed
            return Some(false);
        }
    } else if membership == MembershipState::Leave {
        if target_banned && user_level < ban_level {
            tracing::info!("not enough power to unban");
            return Some(false); // you cannot unban this user
        } else if &target_user_id != event.sender() {
            let kick_level = get_named_level(auth_events, "kick", 50);

            if user_level < kick_level || user_level <= target_level {
                tracing::info!("not enough power to kick user");
                return Some(false); // you do not have the power to kick user
            }
        }
    } else if membership == MembershipState::Ban {
        tracing::debug!(
            "{} < {} || {} <= {}",
            user_level,
            ban_level,
            user_level,
            target_level
        );
        if user_level < ban_level || user_level <= target_level {
            tracing::info!("not enough power to ban");
            return Some(false);
        }
    } else {
        tracing::warn!("unknown membership status");
        // Unknown membership status
        return Some(false);
    }

    Some(true)
}

/// Is the event's sender in the room that they sent the event to.
///
/// A return value of None is not a failure
fn check_event_sender_in_room(
    event: &StateEvent,
    auth_events: &StateMap<StateEvent>,
) -> Option<bool> {
    let mem = auth_events.get(&(EventType::RoomMember, event.sender().to_string()))?;
    // TODO this is check_membership a helper fn in synapse but it does this
    Some(
        mem.deserialize_content::<room::member::MemberEventContent>()
            .ok()?
            .membership
            == MembershipState::Join,
    )
}

/// Is the user allowed to send a specific event.
fn can_send_event(event: &StateEvent, auth_events: &StateMap<StateEvent>) -> Option<bool> {
    let ple = auth_events.get(&(EventType::RoomPowerLevels, "".into()));

    let send_level = get_send_level(event.kind(), event.state_key(), ple);
    let user_level = get_user_power_level(event.sender(), auth_events);

    tracing::debug!(
        "{} snd {} usr {}",
        event.event_id().unwrap().to_string(),
        send_level,
        user_level
    );

    if user_level < send_level {
        return Some(false);
    }

    if let Some(sk) = event.state_key() {
        if sk.starts_with('@') && sk != event.sender().as_str() {
            return Some(false); // permission required to post in this room
        }
    }
    Some(true)
}

/// Confirm that the event sender has the required power levels.
fn check_power_levels(
    room_version: &RoomVersionId,
    power_event: &StateEvent,
    auth_events: &StateMap<StateEvent>,
) -> Option<bool> {
    use itertools::Itertools;

    let key = (power_event.kind(), power_event.state_key().unwrap());

    let current_state = if let Some(current_state) = auth_events.get(&key) {
        current_state
    } else {
        // TODO synapse returns here, shouldn't this be an error ??
        return Some(true);
    };

    let user_content = power_event
        .deserialize_content::<room::power_levels::PowerLevelsEventContent>()
        .unwrap();
    let current_content = current_state
        .deserialize_content::<room::power_levels::PowerLevelsEventContent>()
        .unwrap();

    // validation of users is done in Ruma, synapse for loops validating user_ids and integers here
    tracing::info!("validation of power event finished");

    let user_level = get_user_power_level(power_event.sender(), auth_events);

    let mut user_levels_to_check = btreeset![];
    let old_list = &current_content.users;
    let user_list = &user_content.users;
    for user in old_list.keys().chain(user_list.keys()).dedup() {
        let user: &UserId = user;
        user_levels_to_check.insert(user);
    }

    tracing::debug!("users to check {:?}", user_levels_to_check);

    let mut event_levels_to_check = btreeset![];
    let old_list = &current_content.events;
    let new_list = &user_content.events;
    for ev_id in old_list.keys().chain(new_list.keys()).dedup() {
        let ev_id: &EventType = ev_id;
        event_levels_to_check.insert(ev_id);
    }

    tracing::debug!("events to check {:?}", event_levels_to_check);

    // TODO validate MSC2209 depending on room version check "notifications".
    // synapse does this very differently with the loops (see comments below)
    // but since we have a validated JSON event we can check the levels directly
    // I hope...
    if RoomVersion::new(room_version).limit_notifications_power_levels {
        let old_level: i64 = current_content.notifications.room.into();
        let new_level: i64 = user_content.notifications.room.into();

        let old_level_too_big = old_level > user_level;
        let new_level_too_big = new_level > user_level;
        if old_level_too_big || new_level_too_big {
            tracing::info!("m.room.power_level cannot add ops > than own");
            return Some(false); // cannot add ops greater than own
        }
    }

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
        if user != power_event.sender() && old_level.map(|int| (*int).into()) == Some(user_level) {
            tracing::info!("m.room.power_level cannot remove ops == to own");
            return Some(false); // cannot remove ops level == to own
        }

        let old_level_too_big = old_level.map(|int| (*int).into()) > Some(user_level);
        let new_level_too_big = new_level.map(|int| (*int).into()) > Some(user_level);
        if old_level_too_big || new_level_too_big {
            tracing::info!("m.room.power_level failed to add ops > than own");
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

        let old_level_too_big = old_level.map(|int| (*int).into()) > Some(user_level);
        let new_level_too_big = new_level.map(|int| (*int).into()) > Some(user_level);
        if old_level_too_big || new_level_too_big {
            tracing::info!("m.room.power_level failed to add ops > than own");
            return Some(false); // cannot add ops greater than own
        }
    }

    let levels = [
        "users_default",
        "events_default",
        "state_default",
        "ban",
        "redact",
        "kick",
        "invite",
    ];
    let old_state = serde_json::to_value(old_state).unwrap();
    let new_state = serde_json::to_value(new_state).unwrap();
    for lvl_name in &levels {
        if let Some((old_lvl, new_lvl)) = get_deserialize_levels(&old_state, &new_state, lvl_name) {
            let old_level_too_big = old_lvl > user_level;
            let new_level_too_big = new_lvl > user_level;

            if old_level_too_big || new_level_too_big {
                tracing::info!("cannot add ops > than own");
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
) -> Option<(i64, i64)> {
    Some((
        serde_json::from_value(old.get(name)?.clone()).ok()?,
        serde_json::from_value(new.get(name)?.clone()).ok()?,
    ))
}

/// Does the event redacting come from a user with enough power to redact the given event.
fn check_redaction(
    room_version: &RoomVersionId,
    redaction_event: &StateEvent,
    auth_events: &StateMap<StateEvent>,
) -> Option<RedactAllowed> {
    let user_level = get_user_power_level(redaction_event.sender(), auth_events);
    let redact_level = get_named_level(auth_events, "redact", 50);

    if user_level >= redact_level {
        return Some(RedactAllowed::CanRedact);
    }

    if room_version.is_version_1() {
        if redaction_event.event_id() == redaction_event.redacts() {
            return Some(RedactAllowed::OwnEvent);
        }
    } else {
        // TODO synapse has this line also
        // event.internal_metadata.recheck_redaction = True
        return Some(RedactAllowed::OwnEvent);
    }
    Some(RedactAllowed::No)
}

/// Check that the member event matches `state`.
///
/// This function returns false instead of failing when deserialization fails.
fn check_membership(member_event: Option<&StateEvent>, state: MembershipState) -> bool {
    if let Some(event) = member_event {
        if let Ok(content) =
            serde_json::from_value::<room::member::MemberEventContent>(event.content().clone())
        {
            content.membership == state
        } else {
            false
        }
    } else {
        false
    }
}

fn get_named_level(auth_events: &StateMap<StateEvent>, name: &str, default: i64) -> i64 {
    let power_level_event = auth_events.get(&(EventType::RoomPowerLevels, "".into()));
    if let Some(pl) = power_level_event {
        // TODO do this the right way and deserialize
        if let Some(level) = pl.content().get(name) {
            level.to_string().parse().unwrap_or(default)
        } else {
            0
        }
    } else {
        default
    }
}

fn get_user_power_level(user_id: &UserId, auth_events: &StateMap<StateEvent>) -> i64 {
    if let Some(pl) = auth_events.get(&(EventType::RoomPowerLevels, "".into())) {
        if let Ok(content) = pl.deserialize_content::<room::power_levels::PowerLevelsEventContent>()
        {
            if let Some(level) = content.users.get(user_id) {
                (*level).into()
            } else {
                0
            }
        } else {
            0 // TODO if this fails DB error?
        }
    } else {
        // if no power level event found the creator gets 100 everyone else gets 0
        let key = (EventType::RoomCreate, "".into());
        if let Some(create) = auth_events.get(&key) {
            if let Ok(c) = create.deserialize_content::<room::create::CreateEventContent>() {
                if &c.creator == user_id {
                    100
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        }
    }
}

fn get_send_level(
    e_type: EventType,
    state_key: Option<String>,
    power_lvl: Option<&StateEvent>,
) -> i64 {
    tracing::debug!("{:?} {:?}", e_type, state_key);
    if let Some(ple) = power_lvl {
        if let Ok(content) = serde_json::from_value::<room::power_levels::PowerLevelsEventContent>(
            ple.content().clone(),
        ) {
            let mut lvl: i64 = content
                .events
                .get(&e_type)
                .cloned()
                .unwrap_or(js_int::int!(50))
                .into();
            let state_def: i64 = content.state_default.into();
            let event_def: i64 = content.events_default.into();
            if (state_key.is_some() && state_def > lvl) || event_def > lvl {
                lvl = event_def;
            }
            lvl
        } else {
            50
        }
    } else {
        0
    }
}

fn verify_third_party_invite(_event: &StateEvent, _auth_events: &StateMap<StateEvent>) -> bool {
    unimplemented!("impl third party invites")
}
