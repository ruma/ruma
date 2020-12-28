use std::{collections::BTreeMap, convert::TryFrom, sync::Arc};

use maplit::btreeset;
use ruma::{
    events::{
        pdu::ServerPdu,
        room::{
            self,
            join_rules::JoinRule,
            member::{self, MembershipState},
            power_levels::{self, PowerLevelsEventContent},
        },
        EventType,
    },
    identifiers::{RoomVersionId, UserId},
};

use crate::{state_event::Requester, to_requester, Error, Result, StateMap};

/// For the given event `kind` what are the relevant auth events
/// that are needed to authenticate this `content`.
pub fn auth_types_for_event(
    kind: &EventType,
    sender: &UserId,
    state_key: Option<String>,
    content: serde_json::Value,
) -> Vec<(EventType, Option<String>)> {
    if kind == &EventType::RoomCreate {
        return vec![];
    }

    let mut auth_types = vec![
        (EventType::RoomPowerLevels, Some("".to_string())),
        (EventType::RoomMember, Some(sender.to_string())),
        (EventType::RoomCreate, Some("".to_string())),
    ];

    if kind == &EventType::RoomMember {
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
    incoming_event: &Arc<ServerPdu>,
    prev_event: Option<Arc<ServerPdu>>,
    auth_events: StateMap<Arc<ServerPdu>>,
    current_third_party_invite: Option<Arc<ServerPdu>>,
) -> Result<bool> {
    tracing::info!("auth_check beginning for {}", incoming_event.kind);

    // [synapse] check that all the events are in the same room as `incoming_event`

    // [synapse] do_sig_check check the event has valid signatures for member events

    // TODO do_size_check is false when called by `iterative_auth_check`
    // do_size_check is also mostly accomplished by ruma with the exception of checking event_type,
    // state_key, and json are below a certain size (255 and 65_536 respectively)

    // Implementation of https://matrix.org/docs/spec/rooms/v1#authorization-rules
    //
    // 1. If type is m.room.create:
    if incoming_event.kind == EventType::RoomCreate {
        tracing::info!("start m.room.create check");

        // If it has any previous events, reject
        if !incoming_event.prev_events.is_empty() {
            tracing::warn!("the room creation event had previous events");
            return Ok(false);
        }

        // If the domain of the room_id does not match the domain of the sender, reject
        if incoming_event.room_id.server_name() != incoming_event.sender.server_name() {
            tracing::warn!("creation events server does not match sender");
            return Ok(false); // creation events room id does not match senders
        }

        // If content.room_version is present and is not a recognized version, reject
        if serde_json::from_value::<RoomVersionId>(
            incoming_event
                .content
                .get("room_version")
                .cloned()
                // TODO synapse defaults to version 1
                .unwrap_or_else(|| serde_json::json!("1")),
        )
        .is_err()
        {
            tracing::warn!("invalid room version found in m.room.create event");
            return Ok(false);
        }

        // If content has no creator field, reject
        if incoming_event.content.get("creator").is_none() {
            tracing::warn!("no creator field found in room create content");
            return Ok(false);
        }

        tracing::info!("m.room.create event was allowed");
        return Ok(true);
    }

    /*
    // 2. Reject if auth_events
    // a. auth_events cannot have duplicate keys since it's a BTree
    // b. All entries are valid auth events according to spec
    let expected_auth = auth_types_for_event(
        incoming_event.kind,
        incoming_event.sender(),
        incoming_event.state_key,
        incoming_event.content().clone(),
    );

    dbg!(&expected_auth);

    for ev_key in auth_events.keys() {
        // (b)
        if !expected_auth.contains(ev_key) {
            tracing::warn!("auth_events contained invalid auth event");
            return Ok(false);
        }
    }
    */

    // 3. If event does not have m.room.create in auth_events reject
    if auth_events
        .get(&(EventType::RoomCreate, Some("".into())))
        .is_none()
    {
        tracing::warn!("no m.room.create event in auth chain");

        return Ok(false);
    }

    // [synapse] checks for federation here

    // 4. if type is m.room.aliases
    if incoming_event.kind == EventType::RoomAliases {
        tracing::info!("starting m.room.aliases check");
        // [synapse] adds `&& room_version` "special case aliases auth"

        // [synapse]
        // if event.state_key.unwrap().is_empty() {
        //     tracing::warn!("state_key must be non-empty");
        //     return Ok(false); // and be non-empty state_key (point to a user_id)
        // }

        // If sender's domain doesn't matches state_key, reject
        if incoming_event.state_key != Some(incoming_event.sender.server_name().to_string()) {
            tracing::warn!("state_key does not match sender");
            return Ok(false);
        }

        tracing::info!("m.room.aliases event was allowed");
        return Ok(true);
    }

    if incoming_event.kind == EventType::RoomMember {
        tracing::info!("starting m.room.member check");

        if serde_json::from_value::<room::member::MemberEventContent>(
            incoming_event.content.clone(),
        )
        .is_err()
        {
            tracing::warn!("no membership filed found for m.room.member event content");
            return Ok(false);
        }

        if !valid_membership_change(
            to_requester(incoming_event),
            prev_event,
            current_third_party_invite,
            &auth_events,
        )? {
            return Ok(false);
        }

        tracing::info!("m.room.member event was allowed");
        return Ok(true);
    }

    // If the sender's current membership state is not join, reject
    match check_event_sender_in_room(&incoming_event.sender, &auth_events) {
        Some(true) => {} // sender in room
        Some(false) => {
            tracing::warn!("sender's membership is not join");
            return Ok(false);
        }
        None => {
            tracing::warn!("sender not found in room");
            return Ok(false);
        }
    }

    // Allow if and only if sender's current power level is greater than
    // or equal to the invite level
    if incoming_event.kind == EventType::RoomThirdPartyInvite
        && !can_send_invite(&to_requester(incoming_event), &auth_events)?
    {
        tracing::warn!("sender's cannot send invites in this room");
        return Ok(false);
    }

    // If the event type's required power level is greater than the sender's power level, reject
    // If the event has a state_key that starts with an @ and does not match the sender, reject.
    if !can_send_event(&incoming_event, &auth_events) {
        tracing::warn!("user cannot send event");
        return Ok(false);
    }

    if incoming_event.kind == EventType::RoomPowerLevels {
        tracing::info!("starting m.room.power_levels check");

        if let Some(required_pwr_lvl) =
            check_power_levels(room_version, &incoming_event, &auth_events)
        {
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

    if incoming_event.kind == EventType::RoomRedaction
        && !check_redaction(room_version, incoming_event, &auth_events)?
    {
        return Ok(false);
    }

    tracing::info!("allowing event passed all checks");
    Ok(true)
}

// TODO deserializing the member, power, join_rules event contents is done in conduit
// just before this is called. Could they be passed in?
/// Does the user who sent this member event have required power levels to do so.
///
/// * `user` - Information about the membership event and user making the request.
/// * `prev_event` - The event that occurred immediately before the `user` event or None.
/// * `auth_events` - The set of auth events that relate to a membership event.
/// this is generated by calling `auth_types_for_event` with the membership event and
/// the current State.
pub fn valid_membership_change(
    user: Requester<'_>,
    prev_event: Option<Arc<ServerPdu>>,
    current_third_party_invite: Option<Arc<ServerPdu>>,
    auth_events: &StateMap<Arc<ServerPdu>>,
) -> Result<bool> {
    let state_key = if let Some(s) = user.state_key.as_ref() {
        s
    } else {
        return Err(Error::InvalidPdu("State event requires state_key".into()));
    };

    let content =
        serde_json::from_str::<room::member::MemberEventContent>(&user.content.to_string())?;

    let target_membership = content.membership;

    let target_user_id = UserId::try_from(state_key.as_str())
        .map_err(|e| Error::ConversionError(format!("{}", e)))?;

    let key = (EventType::RoomMember, Some(user.sender.to_string()));
    let sender = auth_events.get(&key);
    let sender_membership =
        sender.map_or(Ok::<_, Error>(member::MembershipState::Leave), |pdu| {
            Ok(
                serde_json::from_value::<room::member::MemberEventContent>(pdu.content.clone())?
                    .membership,
            )
        })?;

    let key = (EventType::RoomMember, Some(target_user_id.to_string()));
    let current = auth_events.get(&key);
    let current_membership =
        current.map_or(Ok::<_, Error>(member::MembershipState::Leave), |pdu| {
            Ok(
                serde_json::from_value::<room::member::MemberEventContent>(pdu.content.clone())?
                    .membership,
            )
        })?;

    let key = (EventType::RoomPowerLevels, Some("".into()));
    let power_levels = auth_events.get(&key).map_or_else(
        || {
            Ok::<_, Error>(power_levels::PowerLevelsEventContent {
                ban: 50.into(),
                events: BTreeMap::new(),
                events_default: 0.into(),
                invite: 50.into(),
                kick: 50.into(),
                redact: 50.into(),
                state_default: 0.into(),
                users: BTreeMap::new(),
                users_default: 0.into(),
                notifications: ruma::events::room::power_levels::NotificationPowerLevels {
                    room: 50.into(),
                },
            })
        },
        |power_levels| {
            serde_json::from_value::<PowerLevelsEventContent>(power_levels.content.clone())
                .map_err(Into::into)
        },
    )?;

    let sender_power = power_levels.users.get(&user.sender).map_or_else(
        || {
            if sender_membership != member::MembershipState::Join {
                None
            } else {
                Some(&power_levels.users_default)
            }
        },
        // If it's okay, wrap with Some(_)
        Some,
    );
    let target_power = power_levels.users.get(&target_user_id).map_or_else(
        || {
            if target_membership != member::MembershipState::Join {
                None
            } else {
                Some(&power_levels.users_default)
            }
        },
        // If it's okay, wrap with Some(_)
        Some,
    );

    let key = (EventType::RoomJoinRules, Some("".into()));
    let join_rules_event = auth_events.get(&key);
    let mut join_rules = JoinRule::Invite;
    if let Some(jr) = join_rules_event {
        join_rules =
            serde_json::from_value::<room::join_rules::JoinRulesEventContent>(jr.content.clone())?
                .join_rule;
    }

    if let Some(prev) = prev_event {
        if prev.kind == EventType::RoomCreate && prev.prev_events.is_empty() {
            return Ok(true);
        }
    }

    Ok(if target_membership == MembershipState::Join {
        if user.sender != &target_user_id {
            false
        } else if let MembershipState::Ban = current_membership {
            false
        } else {
            join_rules == JoinRule::Invite
                && (current_membership == MembershipState::Join
                    || current_membership == MembershipState::Invite)
                || join_rules == JoinRule::Public
        }
    } else if target_membership == MembershipState::Invite {
        // If content has third_party_invite key
        if let Some(tp_id) = content.third_party_invite {
            if current_membership == MembershipState::Ban {
                false
            } else {
                verify_third_party_invite(&user, &tp_id, current_third_party_invite)
            }
        } else if sender_membership != MembershipState::Join
            || current_membership == MembershipState::Join
            || current_membership == MembershipState::Ban
        {
            false
        } else {
            sender_power
                .filter(|&p| p >= &power_levels.invite)
                .is_some()
        }
    } else if target_membership == MembershipState::Leave {
        if user.sender == &target_user_id {
            current_membership == MembershipState::Join
                || current_membership == MembershipState::Invite
        } else if sender_membership != MembershipState::Join
            || current_membership == MembershipState::Ban
                && sender_power.filter(|&p| p < &power_levels.ban).is_some()
        {
            false
        } else {
            sender_power.filter(|&p| p >= &power_levels.kick).is_some()
                && target_power < sender_power
        }
    } else if target_membership == MembershipState::Ban {
        if sender_membership != MembershipState::Join {
            false
        } else {
            sender_power.filter(|&p| p >= &power_levels.ban).is_some()
                && target_power < sender_power
        }
    } else {
        false
    })
}

/// Is the event's sender in the room that they sent the event to.
pub fn check_event_sender_in_room(
    sender: &UserId,
    auth_events: &StateMap<Arc<ServerPdu>>,
) -> Option<bool> {
    let mem = auth_events.get(&(EventType::RoomMember, Some(sender.to_string())))?;
    Some(
        serde_json::from_value::<room::member::MemberEventContent>(mem.content.clone())
            .ok()?
            .membership
            == MembershipState::Join,
    )
}

/// Is the user allowed to send a specific event based on the rooms power levels. Does the event
/// have the correct userId as it's state_key if it's not the "" state_key.
pub fn can_send_event(event: &Arc<ServerPdu>, auth_events: &StateMap<Arc<ServerPdu>>) -> bool {
    let ple = auth_events.get(&(EventType::RoomPowerLevels, Some("".into())));

    let event_type_power_level = get_send_level(&event.kind, event.state_key.clone(), ple);
    let user_level = get_user_power_level(&event.sender, auth_events);

    tracing::debug!(
        "{} ev_type {} usr {}",
        event.event_id.to_string(),
        event_type_power_level,
        user_level
    );

    if user_level < event_type_power_level {
        return false;
    }

    if event
        .state_key
        .as_ref()
        .map_or(false, |k| k.starts_with('@'))
        && event.state_key.as_deref() != Some(event.sender.as_str())
    {
        return false; // permission required to post in this room
    }

    true
}

/// Confirm that the event sender has the required power levels.
pub fn check_power_levels(
    _: &RoomVersionId,
    power_event: &Arc<ServerPdu>,
    auth_events: &StateMap<Arc<ServerPdu>>,
) -> Option<bool> {
    let key = (power_event.kind.clone(), power_event.state_key.clone());
    let current_state = if let Some(current_state) = auth_events.get(&key) {
        current_state
    } else {
        // If there is no previous m.room.power_levels event in the room, allow
        return Some(true);
    };

    // If users key in content is not a dictionary with keys that are valid user IDs
    // with values that are integers (or a string that is an integer), reject.
    let user_content = serde_json::from_value::<room::power_levels::PowerLevelsEventContent>(
        power_event.content.clone(),
    )
    .unwrap();
    let current_content = serde_json::from_value::<room::power_levels::PowerLevelsEventContent>(
        current_state.content.clone(),
    )
    .unwrap();

    // validation of users is done in Ruma, synapse for loops validating user_ids and integers here
    tracing::info!("validation of power event finished");

    let user_level = get_user_power_level(&power_event.sender, auth_events);

    let mut user_levels_to_check = btreeset![];
    let old_list = &current_content.users;
    let user_list = &user_content.users;
    for user in old_list.keys().chain(user_list.keys()) {
        let user: &UserId = user;
        user_levels_to_check.insert(user);
    }

    tracing::debug!("users to check {:?}", user_levels_to_check);

    let mut event_levels_to_check = btreeset![];
    let old_list = &current_content.events;
    let new_list = &user_content.events;
    for ev_id in old_list.keys().chain(new_list.keys()) {
        let ev_id: &EventType = ev_id;
        event_levels_to_check.insert(ev_id);
    }

    tracing::debug!("events to check {:?}", event_levels_to_check);

    // [synapse] validate MSC2209 depending on room version check "notifications".
    // if RoomVersion::new(room_version).limit_notifications_power_levels {
    //     let old_level: i64 = current_content.notifications.room.into();
    //     let new_level: i64 = user_content.notifications.room.into();

    //     let old_level_too_big = old_level > user_level;
    //     let new_level_too_big = new_level > user_level;
    //     if old_level_too_big || new_level_too_big {
    //         tracing::warn!("m.room.power_level cannot add ops > than own");
    //         return Some(false); // cannot add ops greater than own
    //     }
    // }

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
        if user != &power_event.sender && old_level.map(|int| (*int).into()) == Some(user_level) {
            tracing::warn!("m.room.power_level cannot remove ops == to own");
            return Some(false); // cannot remove ops level == to own
        }

        // If the current value is higher than the sender's current power level, reject
        // If the new value is higher than the sender's current power level, reject
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

        // If the current value is higher than the sender's current power level, reject
        // If the new value is higher than the sender's current power level, reject
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
    redaction_event: &Arc<ServerPdu>,
    auth_events: &StateMap<Arc<ServerPdu>>,
) -> Result<bool> {
    let user_level = get_user_power_level(&redaction_event.sender, auth_events);
    let redact_level = get_named_level(auth_events, "redact", 50);

    if user_level >= redact_level {
        tracing::info!("redaction allowed via power levels");
        return Ok(true);
    }

    // FROM SPEC:
    // Redaction events are always accepted (provided the event is allowed by `events` and
    // `events_default` in the power levels). However, servers should not apply or send redaction's
    // to clients until both the redaction event and original event have been seen, and are valid.
    // Servers should only apply redaction's to events where the sender's domains match,
    // or the sender of the redaction has the appropriate permissions per the power levels.

    // version 1 check
    if let RoomVersionId::Version1 = room_version {
        // If the domain of the event_id of the event being redacted is the same as the domain of the event_id of the m.room.redaction, allow
        if redaction_event.event_id.server_name()
            == redaction_event
                .redacts
                .as_ref()
                .and_then(|id| id.server_name())
        {
            tracing::info!("redaction event allowed via room version 1 rules");
            return Ok(true);
        }
    }

    Ok(false)
}

/// Check that the member event matches `state`.
///
/// This function returns false instead of failing when deserialization fails.
pub fn check_membership(member_event: Option<Arc<ServerPdu>>, state: MembershipState) -> bool {
    if let Some(event) = member_event {
        if let Ok(content) =
            serde_json::from_value::<room::member::MemberEventContent>(event.content.clone())
        {
            content.membership == state
        } else {
            false
        }
    } else {
        false
    }
}

/// Can this room federate based on its m.room.create event.
pub fn can_federate(auth_events: &StateMap<Arc<ServerPdu>>) -> bool {
    let creation_event = auth_events.get(&(EventType::RoomCreate, Some("".into())));
    if let Some(ev) = creation_event {
        if let Some(fed) = ev.content.get("m.federate") {
            fed == "true"
        } else {
            false
        }
    } else {
        false
    }
}

/// Helper function to fetch a field, `name`, from a "m.room.power_level" event's content.
/// or return `default` if no power level event is found or zero if no field matches `name`.
pub fn get_named_level(auth_events: &StateMap<Arc<ServerPdu>>, name: &str, default: i64) -> i64 {
    let power_level_event = auth_events.get(&(EventType::RoomPowerLevels, Some("".into())));
    if let Some(pl) = power_level_event {
        // TODO do this the right way and deserialize
        if let Some(level) = pl.content.get(name) {
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
pub fn get_user_power_level(user_id: &UserId, auth_events: &StateMap<Arc<ServerPdu>>) -> i64 {
    if let Some(pl) = auth_events.get(&(EventType::RoomPowerLevels, Some("".into()))) {
        if let Ok(content) = serde_json::from_value::<room::power_levels::PowerLevelsEventContent>(
            pl.content.clone(),
        ) {
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
            if let Ok(c) =
                serde_json::from_value::<room::create::CreateEventContent>(create.content.clone())
            {
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
    e_type: &EventType,
    state_key: Option<String>,
    power_lvl: Option<&Arc<ServerPdu>>,
) -> i64 {
    tracing::debug!("{:?} {:?}", e_type, state_key);
    if let Some(ple) = power_lvl {
        if let Ok(content) = serde_json::from_value::<room::power_levels::PowerLevelsEventContent>(
            ple.content.clone(),
        ) {
            let mut lvl: i64 = content
                .events
                .get(&e_type)
                .cloned()
                .unwrap_or_else(|| ruma::int!(50))
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

/// Check user can send invite.
pub fn can_send_invite(
    event: &Requester<'_>,
    auth_events: &StateMap<Arc<ServerPdu>>,
) -> Result<bool> {
    let user_level = get_user_power_level(event.sender, auth_events);
    let key = (EventType::RoomPowerLevels, Some("".into()));
    let invite_level = auth_events
        .get(&key)
        .map_or_else(
            || Ok::<_, Error>(ruma::int!(50)),
            |power_levels| {
                serde_json::from_value::<PowerLevelsEventContent>(power_levels.content.clone())
                    .map(|pl| pl.invite)
                    .map_err(Into::into)
            },
        )?
        .into();

    Ok(user_level >= invite_level)
}

pub fn verify_third_party_invite(
    event: &Requester<'_>,
    tp_id: &member::ThirdPartyInvite,
    current_third_party_invite: Option<Arc<ServerPdu>>,
) -> bool {
    // 1. check for user being banned happens before this is called
    // checking for mxid and token keys is done by ruma when deserializing

    if event.state_key != Some(tp_id.signed.mxid.to_string()) {
        return false;
    }

    // If there is no m.room.third_party_invite event in the current room state
    // with state_key matching token, reject
    if let Some(current_tpid) = current_third_party_invite {
        if current_tpid.state_key.as_ref() != Some(&tp_id.signed.token) {
            return false;
        }

        if event.sender != &current_tpid.sender {
            return false;
        }

        // If any signature in signed matches any public key in the m.room.third_party_invite event, allow
        if let Ok(tpid_ev) = serde_json::from_value::<
            ruma::events::room::third_party_invite::ThirdPartyInviteEventContent,
        >(current_tpid.content.clone())
        {
            // A list of public keys in the public_keys field
            for key in tpid_ev.public_keys.unwrap_or_default() {
                if key.public_key == tp_id.signed.token {
                    return true;
                }
            }
            // A single public key in the public_key field
            tpid_ev.public_key == tp_id.signed.token
        } else {
            false
        }
    } else {
        false
    }
}
