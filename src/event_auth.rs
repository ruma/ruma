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

use crate::{
    room_version::RoomVersion,
    state_event::{Requester, StateEvent},
    Result, StateMap,
};

/// Represents the 3 event redaction outcomes.
pub enum RedactAllowed {
    /// The event is the users so redaction can take place.
    OwnEvent,
    /// The user can easily redact the event.
    CanRedact,
    /// The user does not have enough power to redact this event.
    No,
}

/// For the given event `kind` what are the relevant auth events
/// that are needed to authenticate this `content`.
pub fn auth_types_for_event(
    kind: EventType,
    sender: &UserId,
    state_key: Option<String>,
    content: serde_json::Value,
) -> Vec<(EventType, Option<String>)> {
    if kind == EventType::RoomCreate {
        return vec![];
    }

    let mut auth_types = vec![
        (EventType::RoomPowerLevels, Some("".to_string())),
        (EventType::RoomMember, Some(sender.to_string())),
        (EventType::RoomCreate, Some("".to_string())),
    ];

    if kind == EventType::RoomMember {
        if let Ok(content) = serde_json::from_value::<room::member::MemberEventContent>(content) {
            if [MembershipState::Join, MembershipState::Invite].contains(&content.membership) {
                let key = (EventType::RoomJoinRules, Some("".into()));
                if !auth_types.contains(&key) {
                    auth_types.push(key)
                }
            }

            // TODO what when we don't find a state_key
            let key = (EventType::RoomMember, state_key);
            if !auth_types.contains(&key) {
                auth_types.push(key)
            }

            if content.membership == MembershipState::Invite {
                if let Some(t_id) = content.third_party_invite {
                    let key = (EventType::RoomThirdPartyInvite, Some(t_id.signed.token));
                    if !auth_types.contains(&key) {
                        auth_types.push(key)
                    }
                }
            }
        }
    }

    auth_types
}

/// Authenticate the incoming `event`. The steps of authentication are:
/// * check that the event is being authenticated for the correct room
/// * check that the events signatures are valid
/// * then there are checks for specific event types
pub fn auth_check(
    room_version: &RoomVersionId,
    event: &StateEvent,
    auth_events: StateMap<StateEvent>,
    do_sig_check: bool,
) -> Result<bool> {
    tracing::info!("auth_check beginning for {}", event.event_id().as_str());

    // don't let power from other rooms be used
    for auth_event in auth_events.values() {
        if auth_event.room_id() != event.room_id() {
            tracing::warn!("found auth event that did not match event's room_id");
            return Ok(false);
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

        // check the event has been signed by the domain of the sender
        if event.signatures().get(sender_domain).is_none() && !is_invite_via_3pid {
            tracing::warn!("event not signed by sender's server");
            return Ok(false);
        }

        if event.room_version() == RoomVersionId::Version1
            && event
                .signatures()
                .get(event.event_id().server_name().unwrap())
                .is_none()
        {
            tracing::warn!("event not signed by event_id's server");
            return Ok(false);
        }
    }

    // TODO do_size_check is false when called by `iterative_auth_check`
    // do_size_check is also mostly accomplished by ruma with the exception of checking event_type,
    // state_key, and json are below a certain size (255 and 65536 respectively)

    // Implementation of https://matrix.org/docs/spec/rooms/v1#authorization-rules
    //
    // 1. If type is m.room.create:
    if event.kind() == EventType::RoomCreate {
        tracing::info!("start m.room.create check");

        // domain of room_id must match domain of sender.
        if event.room_id().map(|id| id.server_name()) != Some(event.sender().server_name()) {
            tracing::warn!("creation events server does not match sender");
            return Ok(false); // creation events room id does not match senders
        }

        // if content.room_version is present and is not a valid version
        if serde_json::from_value::<RoomVersionId>(
            event
                .content()
                .get("room_version")
                .cloned()
                // synapse defaults to version 1
                .unwrap_or_else(|| serde_json::json!("1")),
        )
        .is_err()
        {
            tracing::warn!("invalid room version found in m.room.create event");
            return Ok(false);
        }

        tracing::info!("m.room.create event was allowed");
        return Ok(true);
    }

    // 3. If event does not have m.room.create in auth_events reject.
    if auth_events
        .get(&(EventType::RoomCreate, Some("".into())))
        .is_none()
    {
        tracing::warn!("no m.room.create event in auth chain");

        return Ok(false);
    }

    // check for m.federate
    if event.room_id().map(|id| id.server_name()) != Some(event.sender().server_name()) {
        tracing::info!("checking federation");

        if !can_federate(&auth_events) {
            tracing::warn!("federation not allowed");

            return Ok(false);
        }
    }

    // 4. if type is m.room.aliases
    if event.kind() == EventType::RoomAliases {
        tracing::info!("starting m.room.aliases check");
        // TODO && room_version "special case aliases auth" ??
        if event.state_key().is_none() {
            tracing::warn!("no state_key field found for event");
            return Ok(false); // must have state_key
        }
        if event.state_key().unwrap().is_empty() {
            tracing::warn!("state_key must be non-empty");
            return Ok(false); // and be non-empty state_key (point to a user_id)
        }

        if event.state_key() != Some(event.sender().to_string()) {
            tracing::warn!("no state_key field found for event");
            return Ok(false);
        }

        tracing::info!("m.room.aliases event was allowed");
        return Ok(true);
    }

    if event.kind() == EventType::RoomMember {
        tracing::info!("starting m.room.member check");

        if !is_membership_change_allowed(event.to_requester(), &auth_events)? {
            return Ok(false);
        }

        tracing::info!("m.room.member event was allowed");
        return Ok(true);
    }

    if let Ok(in_room) = check_event_sender_in_room(event, &auth_events) {
        if !in_room {
            tracing::warn!("sender not in room");
            return Ok(false);
        }
    } else {
        tracing::warn!("sender not in room");
        return Ok(false);
    }

    // Special case to allow m.room.third_party_invite events where ever
    // a user is allowed to issue invites
    if event.kind() == EventType::RoomThirdPartyInvite {
        // TODO impl this
        unimplemented!("third party invite")
    }

    if !can_send_event(event, &auth_events)? {
        tracing::warn!("user cannot send event");
        return Ok(false);
    }

    if event.kind() == EventType::RoomPowerLevels {
        tracing::info!("starting m.room.power_levels check");
        if let Some(required_pwr_lvl) = check_power_levels(room_version, event, &auth_events) {
            if !required_pwr_lvl {
                tracing::warn!("power level was not allowed");
                return Ok(false);
            }
        } else {
            tracing::warn!("power level was not allowed");
            return Ok(false);
        }
        tracing::info!("power levels event allowed");
    }

    if event.kind() == EventType::RoomRedaction {
        if let RedactAllowed::No = check_redaction(room_version, event, &auth_events)? {
            return Ok(false);
        }
    }

    tracing::info!("allowing event passed all checks");
    Ok(true)
}

// synapse has an `event: &StateEvent` param but it's never used
/// Can this room federate based on its m.room.create event.
pub fn can_federate(auth_events: &StateMap<StateEvent>) -> bool {
    let creation_event = auth_events.get(&(EventType::RoomCreate, Some("".into())));
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

/// Does the user who sent this member event have required power levels to do so.
///
/// If called on it's own the following must be true:
/// - there must be a valid state_key in `user`
/// - there must be a membership key in `user.content` i.e. the event is of type "m.room.member"
pub fn is_membership_change_allowed(
    user: Requester<'_>,
    auth_events: &StateMap<StateEvent>,
) -> Result<bool> {
    let content =
        // TODO return error
        serde_json::from_str::<room::member::MemberEventContent>(&user.content.to_string())?;

    let membership = content.membership;

    // If the only previous event is an m.room.create and the state_key is the creator, allow
    if user.prev_event_ids.len() == 1 && membership == MembershipState::Join {
        if let Some(create) = auth_events.get(&(EventType::RoomCreate, Some("".into()))) {
            if let Ok(create_ev) = create.deserialize_content::<room::create::CreateEventContent>()
            {
                if user.state_key == Some(create_ev.creator.to_string()) {
                    tracing::debug!("m.room.member event allowed via m.room.create");
                    return Ok(true);
                }
            }
        }
    }

    let target_user_id = UserId::try_from(user.state_key.as_deref().unwrap()).unwrap();

    let key = (EventType::RoomMember, Some(user.sender.to_string()));
    let caller = auth_events.get(&key);

    let caller_in_room = caller.is_some() && check_membership(caller, MembershipState::Join);
    let caller_invited = caller.is_some() && check_membership(caller, MembershipState::Invite);

    let key = (EventType::RoomMember, Some(target_user_id.to_string()));
    let target = auth_events.get(&key);

    let target_in_room = target.is_some() && check_membership(target, MembershipState::Join);
    let target_banned = target.is_some() && check_membership(target, MembershipState::Ban);

    let key = (EventType::RoomJoinRules, Some("".to_string()));
    let join_rules_event = auth_events.get(&key);

    let mut join_rule = JoinRule::Invite;
    if let Some(jr) = join_rules_event {
        join_rule = jr
            .deserialize_content::<room::join_rules::JoinRulesEventContent>()?
            .join_rule;
    }

    let user_level = get_user_power_level(user.sender, auth_events);
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
            "event.user_id": user.sender,
        }))
        .unwrap(),
    );

    if membership == MembershipState::Invite && content.third_party_invite.is_some() {
        // TODO this is unimpled
        if !verify_third_party_invite(&user, auth_events) {
            tracing::warn!("not invited to this room",);
            return Ok(false);
        }
        if target_banned {
            tracing::warn!("banned from this room",);
            return Ok(false);
        }
        tracing::info!("invite succeded");
        return Ok(true);
    }

    if membership == MembershipState::Invite {
        if !caller_in_room {
            tracing::warn!("invite sender not in room they are inviting user to");
            return Ok(false);
        }

        if target_banned {
            tracing::warn!("target has been banned");
            return Ok(false);
        } else if target_in_room {
            tracing::warn!("already in room");
            return Ok(false); // already in room
        } else {
            let invite_level = get_named_level(auth_events, "invite", 0);
            if user_level < invite_level {
                return Ok(false);
            }
        }
    } else if membership == MembershipState::Join {
        if user.sender != &target_user_id {
            tracing::warn!("cannot force another user to join");
            return Ok(false); // cannot force another user to join
        } else if target_banned {
            tracing::warn!("cannot join when banned");
            return Ok(false); // cannot joined when banned
        } else if join_rule == JoinRule::Public {
            tracing::info!("join rule public")
        // pass
        } else if join_rule == JoinRule::Invite {
            if !caller_in_room && !caller_invited {
                tracing::warn!("user has not been invited to this room");
                return Ok(false); // you are not invited to this room
            }
        } else {
            tracing::warn!("the join rule is Private or yet to be spec'ed by Matrix");
            // synapse has 2 TODO's may_join list and private rooms

            // the join_rule is Private or Knock which means it is not yet spec'ed
            return Ok(false);
        }
    } else if membership == MembershipState::Leave {
        if !caller_in_room {
            tracing::warn!("sender not in room they are leaving");
            return Ok(false);
        }

        if target_banned && user_level < ban_level {
            tracing::warn!("not enough power to unban");
            return Ok(false); // you cannot unban this user
        } else if &target_user_id != user.sender {
            let kick_level = get_named_level(auth_events, "kick", 50);

            if user_level < kick_level || user_level <= target_level {
                tracing::warn!("not enough power to kick user");
                return Ok(false); // you do not have the power to kick user
            }
        }
    } else if membership == MembershipState::Ban {
        if !caller_in_room {
            tracing::warn!("ban sender not in room they are banning user from");
            return Ok(false);
        }

        tracing::debug!(
            "{} < {} || {} <= {}",
            user_level,
            ban_level,
            user_level,
            target_level
        );

        if user_level < ban_level || user_level <= target_level {
            tracing::warn!("not enough power to ban");
            return Ok(false);
        }
    } else {
        tracing::warn!("unknown membership status");
        // Unknown membership status
        return Ok(false);
    }

    Ok(true)
}

/// Is the event's sender in the room that they sent the event to.
///
/// A return value of None is not a failure
pub fn check_event_sender_in_room(
    event: &StateEvent,
    auth_events: &StateMap<StateEvent>,
) -> Result<bool> {
    let mem = auth_events
        .get(&(EventType::RoomMember, Some(event.sender().to_string())))
        .ok_or_else(|| crate::Error::NotFound("Authe event was not found".into()))?;
    // TODO this is check_membership a helper fn in synapse but it does this
    Ok(mem
        .deserialize_content::<room::member::MemberEventContent>()?
        .membership
        == MembershipState::Join)
}

/// Is the user allowed to send a specific event based on the rooms power levels.
pub fn can_send_event(event: &StateEvent, auth_events: &StateMap<StateEvent>) -> Result<bool> {
    let ple = auth_events.get(&(EventType::RoomPowerLevels, Some("".into())));

    let send_level = get_send_level(event.kind(), event.state_key(), ple);
    let user_level = get_user_power_level(event.sender(), auth_events);

    tracing::debug!(
        "{} snd {} usr {}",
        event.event_id().to_string(),
        send_level,
        user_level
    );

    if user_level < send_level {
        return Ok(false);
    }

    if let Some(sk) = event.state_key() {
        if sk.starts_with('@') && sk != event.sender().as_str() {
            return Ok(false); // permission required to post in this room
        }
    }
    Ok(true)
}

/// Confirm that the event sender has the required power levels.
pub fn check_power_levels(
    room_version: &RoomVersionId,
    power_event: &StateEvent,
    auth_events: &StateMap<StateEvent>,
) -> Option<bool> {
    use itertools::Itertools;

    let key = (power_event.kind(), power_event.state_key());

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
            tracing::warn!("m.room.power_level cannot add ops > than own");
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
            tracing::warn!("m.room.power_level cannot remove ops == to own");
            return Some(false); // cannot remove ops level == to own
        }

        let old_level_too_big = old_level.map(|int| (*int).into()) > Some(user_level);
        let new_level_too_big = new_level.map(|int| (*int).into()) > Some(user_level);
        if old_level_too_big || new_level_too_big {
            tracing::warn!("m.room.power_level failed to add ops > than own");
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
            tracing::warn!("m.room.power_level failed to add ops > than own");
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
                tracing::warn!("cannot add ops > than own");
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
pub fn check_redaction(
    room_version: &RoomVersionId,
    redaction_event: &StateEvent,
    auth_events: &StateMap<StateEvent>,
) -> Result<RedactAllowed> {
    let user_level = get_user_power_level(redaction_event.sender(), auth_events);
    let redact_level = get_named_level(auth_events, "redact", 50);

    if user_level >= redact_level {
        return Ok(RedactAllowed::CanRedact);
    }

    if let RoomVersionId::Version1 = room_version {
        // are the redacter and redactee in the same domain
        if Some(redaction_event.event_id().server_name())
            == redaction_event.redacts().map(|id| id.server_name())
        {
            return Ok(RedactAllowed::OwnEvent);
        }
    } else {
        // TODO synapse has this line also
        // event.internal_metadata.recheck_redaction = True
        return Ok(RedactAllowed::OwnEvent);
    }
    Ok(RedactAllowed::No)
}

/// Check that the member event matches `state`.
///
/// This function returns false instead of failing when deserialization fails.
pub fn check_membership(member_event: Option<&StateEvent>, state: MembershipState) -> bool {
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

/// Helper function to fetch a field, `name`, from a "m.room.power_level" event's content.
/// or return `default` if no power level event is found or zero if no field matches `name`.
pub fn get_named_level(auth_events: &StateMap<StateEvent>, name: &str, default: i64) -> i64 {
    let power_level_event = auth_events.get(&(EventType::RoomPowerLevels, Some("".into())));
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

/// Helper function to fetch a users default power level from a "m.room.power_level" event's `users`
/// object.
pub fn get_user_power_level(user_id: &UserId, auth_events: &StateMap<StateEvent>) -> i64 {
    if let Some(pl) = auth_events.get(&(EventType::RoomPowerLevels, Some("".into()))) {
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
        let key = (EventType::RoomCreate, Some("".into()));
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

/// Helper function to fetch the power level needed to send an event of type
/// `e_type` based on the rooms "m.room.power_level" event.
pub fn get_send_level(
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
                .unwrap_or_else(|| js_int::int!(50))
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

/// TODO this is unimplemented
pub fn verify_third_party_invite(
    _event: &Requester<'_>,
    _auth_events: &StateMap<StateEvent>,
) -> bool {
    unimplemented!("impl third party invites")
}
