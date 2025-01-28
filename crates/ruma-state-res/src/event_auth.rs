use std::{borrow::Borrow, collections::BTreeSet};

use js_int::{int, Int};
use ruma_common::{serde::Raw, OwnedUserId, RoomVersionId, UserId};
use ruma_events::room::{
    create::RoomCreateEventContent,
    member::{MembershipState, ThirdPartyInvite},
    power_levels::RoomPowerLevelsEventContent,
};
use serde::{de::IgnoredAny, Deserialize};
use serde_json::{from_str as from_json_str, value::RawValue as RawJsonValue};
use tracing::{debug, error, info, instrument, trace, warn};

mod room_member;
#[cfg(test)]
mod tests;

use self::room_member::check_room_member;
use crate::{
    events::{
        deserialize_power_levels, deserialize_power_levels_content_fields,
        deserialize_power_levels_content_invite, deserialize_power_levels_content_redact,
    },
    room_version::RoomVersion,
    Event, Result, StateEventType, TimelineEventType,
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
    fetch_state: impl Fn(&StateEventType, &str) -> Option<E>,
) -> Result<bool> {
    debug!("starting auth check");

    // Since v1, if type is m.room.create:
    if *incoming_event.event_type() == TimelineEventType::RoomCreate {
        return check_room_create(incoming_event, room_version);
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

    let sender = incoming_event.sender();

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

    // Since v1, if type is m.room.member:
    if *incoming_event.event_type() == TimelineEventType::RoomMember {
        return check_room_member(incoming_event, room_version, room_create_event, fetch_state);
    }

    // Since v1, if the sender's current membership state is not join, reject.
    let sender_member_event = fetch_state(&StateEventType::RoomMember, sender.as_str());
    let sender_member_event = match sender_member_event {
        Some(mem) => mem,
        None => {
            warn!("sender not found in room");
            return Ok(false);
        }
    };

    let sender_membership_event_content: RoomMemberContentFields =
        from_json_str(sender_member_event.content().get())?;
    let membership_state =
        sender_membership_event_content.membership.map(|m| m.deserialize()).transpose()?;

    if !matches!(membership_state, Some(MembershipState::Join)) {
        warn!("sender's membership is not join");
        return Ok(false);
    }

    let power_levels_event = fetch_state(&StateEventType::RoomPowerLevels, "");

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

    // Since v1, if the event type's required power level is greater than the sender's power level,
    // reject.
    let event_type_power_level = get_send_level(
        incoming_event.event_type(),
        incoming_event.state_key(),
        power_levels_event.as_ref(),
    );
    if sender_power_level < event_type_power_level {
        warn!(event_type = %incoming_event.event_type(), "user doesn't have enough power to send event type");
        return Ok(false);
    }

    // Since v1, if the event has a state_key that starts with an @ and does not match the sender,
    // reject.
    if incoming_event.state_key().is_some_and(|k| k.starts_with('@'))
        && incoming_event.state_key() != Some(incoming_event.sender().as_str())
    {
        warn!("sender cannot send state event with another user's ID");
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

/// Check whether the given event passes the `m.room.create` authorization rules.
fn check_room_create(room_create_event: impl Event, room_version: &RoomVersion) -> Result<bool> {
    #[derive(Deserialize)]
    struct RoomCreateContentFields {
        room_version: Option<Raw<RoomVersionId>>,
        creator: Option<Raw<IgnoredAny>>,
    }

    debug!("start m.room.create check");

    // Since v1, if it has any previous events, reject.
    if room_create_event.prev_events().next().is_some() {
        warn!("the room creation event had previous events");
        return Ok(false);
    }

    // Since v1, if the domain of the room_id does not match the domain of the sender, reject.
    let Some(room_id_server_name) = room_create_event.room_id().server_name() else {
        warn!("room ID has no servername");
        return Ok(false);
    };

    if room_id_server_name != room_create_event.sender().server_name() {
        warn!("servername of room ID does not match servername of sender");
        return Ok(false);
    }

    // Since v1, if `content.room_version` is present and is not a recognized version, reject.
    //
    // FIXME: this only checks if we can deserialize to `RoomVersionId` which accepts any
    // string. We should check if the version is actually supported, i.e. if we have a
    // `RoomVersion` for it. But we already take a `RoomVersion` as a parameter so this was
    // already checked before?
    let content: RoomCreateContentFields = from_json_str(room_create_event.content().get())?;
    if content.room_version.map(|v| v.deserialize().is_err()).unwrap_or(false) {
        warn!("invalid room version found in m.room.create event");
        return Ok(false);
    }

    // v1-v10, if content has no creator field, reject.
    if !room_version.use_room_create_sender && content.creator.is_none() {
        warn!("no creator field found in m.room.create content");
        return Ok(false);
    }

    // Otherwise, allow.
    info!("m.room.create event was allowed");
    Ok(true)
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
