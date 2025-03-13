use std::{
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet},
};

use js_int::Int;
use ruma_common::UserId;
use ruma_events::room::member::MembershipState;
use serde_json::value::RawValue as RawJsonValue;
use tracing::{debug, info, instrument, warn};

mod room_member;
#[cfg(test)]
mod tests;

use self::room_member::check_room_member;
use crate::{
    events::{
        member::{RoomMemberEventContent, RoomMemberEventOptionExt},
        power_levels::{RoomPowerLevelsEventOptionExt, RoomPowerLevelsIntField},
        JoinRule, RoomCreateEvent, RoomJoinRulesEvent, RoomMemberEvent, RoomPowerLevelsEvent,
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
) -> std::result::Result<Vec<(StateEventType, String)>, String> {
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
        let Some(state_key) = state_key else {
            return Err("missing `state_key` field for `m.room.member` event".to_owned());
        };
        let content = RoomMemberEventContent::new(content);
        let membership = content.membership()?;

        if matches!(
            membership,
            MembershipState::Join | MembershipState::Invite | MembershipState::Knock
        ) {
            // If membership is join, invite or knock, we need `m.room.join_rules`.
            let key = (StateEventType::RoomJoinRules, "".to_owned());
            if !auth_types.contains(&key) {
                auth_types.push(key);
            }

            let join_authorised_via_users_server = content.join_authorised_via_users_server()?;
            if let Some(user_id) = join_authorised_via_users_server {
                // If `join_authorised_via_users_server` is present, and the room
                // version supports restricted rooms, we need `m.room.member` with the
                // matching state_key.
                //
                // FIXME: We need to check that the room version supports restricted rooms
                // too.
                let key = (StateEventType::RoomMember, user_id.to_string());
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
            let third_party_invite = content.third_party_invite()?;

            if let Some(third_party_invite) = third_party_invite {
                // If membership is invite and `third_party_invite` is present, we need
                // `m.room.third_party_invite` with the state_key matching
                // `third_party_invite.signed.token`.
                let token = third_party_invite.token()?.to_owned();
                let key = (StateEventType::RoomThirdPartyInvite, token);
                if !auth_types.contains(&key) {
                    auth_types.push(key);
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
        let room_create_event = RoomCreateEvent::new(incoming_event);

        return check_room_create(room_create_event, room_version)
            .map(|_| true)
            .map_err(Error::custom);
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

    let room_create_event = fetch_state.room_create_event().map_err(Error::custom)?;

    // Since v1, if there is no m.room.create event among the entries, reject.
    if !incoming_event.auth_events().any(|id| id.borrow() == room_create_event.event_id().borrow())
    {
        return Err(Error::custom("no `m.room.create` event in auth events"));
    }

    // Since v1, if the create event content has the field m.federate set to false and the sender
    // domain of the event does not match the sender domain of the create event, reject.
    let federate = room_create_event.federate().map_err(Error::custom)?;
    if !federate
        && room_create_event.sender().server_name() != incoming_event.sender().server_name()
    {
        return Err(Error::custom("room is not federated and event's sender domain does not match `m.room.create` event's sender domain"));
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
        let room_member_event = RoomMemberEvent::new(incoming_event);
        return check_room_member(room_member_event, room_version, room_create_event, fetch_state)
            .map(|_| true)
            .map_err(Error::custom);
    }

    // Since v1, if the sender's current membership state is not join, reject.
    let sender_membership = fetch_state.user_membership(sender).map_err(Error::custom)?;

    if sender_membership != MembershipState::Join {
        warn!("sender's membership is not `join`");
        return Ok(false);
    }

    let creator = room_create_event.creator(room_version).map_err(Error::custom)?;
    let current_room_power_levels_event = fetch_state.room_power_levels_event();

    let sender_power_level = current_room_power_levels_event
        .user_power_level(sender, &creator, room_version)
        .map_err(Error::custom)?;

    // Since v1, if type is m.room.third_party_invite:
    if *incoming_event.event_type() == TimelineEventType::RoomThirdPartyInvite {
        // Since v1, allow if and only if sender's current power level is greater than
        // or equal to the invite level.
        let invite_power_level = current_room_power_levels_event
            .get_as_int_or_default(RoomPowerLevelsIntField::Invite, room_version)
            .map_err(Error::custom)?;

        if sender_power_level < invite_power_level {
            return Err(Error::custom(
                "sender does not have enough power to send invites in this room",
            ));
        }

        info!("m.room.third_party_invite event was allowed");
        return Ok(true);
    }

    // Since v1, if the event type's required power level is greater than the sender's power level,
    // reject.
    let event_type_power_level = current_room_power_levels_event
        .event_power_level(incoming_event.event_type(), incoming_event.state_key(), room_version)
        .map_err(Error::custom)?;
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
        let room_power_levels_event = RoomPowerLevelsEvent::new(incoming_event);
        return check_room_power_levels(
            room_power_levels_event,
            current_room_power_levels_event,
            room_version,
            sender_power_level,
        )
        .map(|_| true)
        .map_err(Error::custom);
    }

    // v1-v2, if type is m.room.redaction:
    if room_version.extra_redaction_checks
        && *incoming_event.event_type() == TimelineEventType::RoomRedaction
    {
        return check_room_redaction(
            incoming_event,
            current_room_power_levels_event,
            room_version,
            sender_power_level,
        );
    }

    // Otherwise, allow.
    info!("allowing event passed all checks");
    Ok(true)
}

/// Check whether the given event passes the `m.room.create` authorization rules.
fn check_room_create(
    room_create_event: RoomCreateEvent<impl Event>,
    room_version: &RoomVersion,
) -> std::result::Result<(), String> {
    debug!("start `m.room.create` check");

    // Since v1, if it has any previous events, reject.
    if room_create_event.prev_events().next().is_some() {
        return Err("`m.room.create` event cannot have previous events".into());
    }

    // Since v1, if the domain of the room_id does not match the domain of the sender, reject.
    let Some(room_id_server_name) = room_create_event.room_id().server_name() else {
        return Err(
            "invalid `room_id` field in `m.room.create` event: could not parse server name".into(),
        );
    };

    if room_id_server_name != room_create_event.sender().server_name() {
        return Err("invalid `room_id` field in `m.room.create` event: server name does not match sender's server name".into());
    }

    // Since v1, if `content.room_version` is present and is not a recognized version, reject.
    //
    // FIXME: this only checks if we can deserialize to `RoomVersionId` which accepts any
    // string. We should check if the version is actually supported, i.e. if we have a
    // `RoomVersion` for it. But we already take a `RoomVersion` as a parameter so this was
    // already checked before?
    room_create_event.room_version()?;

    // v1-v10, if content has no creator field, reject.
    if !room_version.use_room_create_sender && !room_create_event.has_creator()? {
        return Err("missing `creator` field in `m.room.create` event".into());
    }

    // Otherwise, allow.
    info!("`m.room.create` event was allowed");
    Ok(())
}

/// Check whether the given event passes the `m.room.power_levels` authorization rules.
fn check_room_power_levels(
    room_power_levels_event: RoomPowerLevelsEvent<impl Event>,
    current_room_power_levels_event: Option<RoomPowerLevelsEvent<impl Event>>,
    room_version: &RoomVersion,
    sender_power_level: Int,
) -> std::result::Result<(), String> {
    debug!("starting m.room.power_levels check");

    // FIXME: the authorization rules do not say to check the state key, which is weird because we
    // couldn't get the previous room power levels if it didn't have a state key. Instead of
    // checking for an empty string, Synapse fetches the previous power levels with the same state
    // key, which might be more correct.
    match room_power_levels_event.state_key() {
        Some("") => {}
        Some(_) => {
            return Err("`m.room.power_levels` event has non-empty `state_key`".to_owned());
        }
        None => {
            return Err("missing `state_key` for `m.room.power_levels` event".to_owned());
        }
    }

    // Since v10, if any of the properties users_default, events_default, state_default, ban,
    // redact, kick, or invite in content are present and not an integer, reject.
    let new_int_fields = room_power_levels_event.int_fields_map(room_version)?;

    // Since v10, if either of the properties events or notifications in content are present and not
    // a dictionary with values that are integers, reject.
    let new_events = room_power_levels_event.events(room_version)?;
    let new_notifications = room_power_levels_event.notifications(room_version)?;

    // v1-v9, If the users property in content is not an object with keys that are valid user IDs
    // with values that are integers (or a string that is an integer), reject.
    // Since v10, if the users property in content is not an object with keys that are valid user
    // IDs with values that are integers, reject.
    let new_users = room_power_levels_event.users(room_version)?;

    debug!("validation of power event finished");

    // Since v1, if there is no previous m.room.power_levels event in the room, allow.
    let Some(current_room_power_levels_event) = current_room_power_levels_event else {
        info!("initial m.room.power_levels event allowed");
        return Ok(());
    };

    // Since v1, for the properties users_default, events_default, state_default, ban, redact, kick,
    // invite check if they were added, changed or removed. For each found alteration:
    for field in RoomPowerLevelsIntField::ALL {
        let current_power_level =
            current_room_power_levels_event.get_as_int(*field, room_version)?;
        let new_power_level = new_int_fields.get(field).copied();

        if current_power_level == new_power_level {
            continue;
        }

        // Since v1, if the current value is higher than the sender’s current power level,
        // reject.
        let current_power_level_too_big =
            current_power_level.unwrap_or_else(|| field.default_value()) > sender_power_level;
        // Since v1, if the new value is higher than the sender’s current power level, reject.
        let new_power_level_too_big =
            new_power_level.unwrap_or_else(|| field.default_value()) > sender_power_level;

        if current_power_level_too_big || new_power_level_too_big {
            return Err(format!(
                "sender does not have enough power to change the power level of `{field}`"
            ));
        }
    }

    // Since v1, for each entry being added to, or changed in, the events property:
    // - Since v1, if the new value is higher than the sender's current power level, reject.
    let current_events = current_room_power_levels_event.events(room_version)?;
    check_power_level_maps(
        current_events.as_ref(),
        new_events.as_ref(),
        &sender_power_level,
        |_, current_power_level| {
            // Since v1, for each entry being changed in, or removed from, the events property:
            // - Since v1, if the current value is higher than the sender's current power level,
            //   reject.
            current_power_level > sender_power_level
        },
        |ev_type| {
            format!(
            "sender does not have enough power to change the `{ev_type}` event type power level"
        )
        },
    )?;

    // Since v6, for each entry being added to, or changed in, the notifications property:
    // - Since v6, if the new value is higher than the sender's current power level, reject.
    if room_version.limit_notifications_power_levels {
        let current_notifications = current_room_power_levels_event.notifications(room_version)?;
        check_power_level_maps(
            current_notifications.as_ref(),
            new_notifications.as_ref(),
            &sender_power_level,
            |_, current_power_level| {
                // Since v6, for each entry being changed in, or removed from, the notifications
                // property:
                // - Since v6, if the current value is higher than the sender's current power level,
                //   reject.
                current_power_level > sender_power_level
            },
            |key| {
                format!(
                "sender does not have enough power to change the `{key}` notification power level"
            )
            },
        )?;
    }

    // Since v1, for each entry being added to, or changed in, the users property:
    // - Since v1, if the new value is greater than the sender’s current power level, reject.
    let current_users = current_room_power_levels_event.users(room_version)?;
    check_power_level_maps(
        current_users,
        new_users,
        &sender_power_level,
        |user_id, current_power_level| {
            // Since v1, for each entry being changed in, or removed from, the users property, other
            // than the sender’s own entry:
            // - Since v1, if the current value is greater than or equal to the sender’s current
            //   power level, reject.
            user_id != room_power_levels_event.sender() && current_power_level >= sender_power_level
        },
        |user_id| format!("sender does not have enough power to change `{user_id}`'s  power level"),
    )?;

    // Otherwise, allow.
    info!("m.room.power_levels event allowed");
    Ok(())
}

/// Check the power levels changes between the current and the new maps.
///
/// # Arguments
///
/// * `current`: the map with the current power levels.
/// * `new`: the map with the new power levels.
/// * `sender_power_level`: the power level of the sender of the new map.
/// * `reject_current_power_level_change_fn`: the function to check if a power level change or
///   removal must be rejected given its current value.
///
///   The arguments to the method are the key of the power level and the current value of the power
///   level. It must return `true` if the change or removal is rejected.
///
///   Note that another check is done after this one to check if the change is allowed given the new
///   value of the power level.
/// * `error_fn`: the function to generate an error when the change for the given key is not
///   allowed.
fn check_power_level_maps<K: Ord>(
    current: Option<&BTreeMap<K, Int>>,
    new: Option<&BTreeMap<K, Int>>,
    sender_power_level: &Int,
    reject_current_power_level_change_fn: impl FnOnce(&K, Int) -> bool + Copy,
    error_fn: impl FnOnce(&K) -> String,
) -> std::result::Result<(), String> {
    let keys_to_check = current
        .iter()
        .flat_map(|m| m.keys())
        .chain(new.iter().flat_map(|m| m.keys()))
        .collect::<BTreeSet<_>>();

    for key in keys_to_check {
        let current_power_level = current.as_ref().and_then(|m| m.get(key));
        let new_power_level = new.as_ref().and_then(|m| m.get(key));

        if current_power_level == new_power_level {
            continue;
        }

        // For each entry being changed in, or removed from, the property.
        let current_power_level_change_rejected = current_power_level
            .is_some_and(|power_level| reject_current_power_level_change_fn(key, *power_level));

        // For each entry being added to, or changed in, the property:
        // - If the new value is higher than the sender's current power level, reject.
        let new_power_level_too_big = new_power_level > Some(sender_power_level);

        if current_power_level_change_rejected || new_power_level_too_big {
            return Err(error_fn(key));
        }
    }

    Ok(())
}

/// Check whether the given event passes the `m.room.redaction` authorization rules.
fn check_room_redaction(
    room_redaction_event: impl Event,
    current_room_power_levels_event: Option<RoomPowerLevelsEvent<impl Event>>,
    room_version: &RoomVersion,
    sender_level: Int,
) -> Result<bool> {
    let redact_level = current_room_power_levels_event
        .get_as_int_or_default(RoomPowerLevelsIntField::Redact, room_version)
        .map_err(Error::custom)?;

    // v1-v2, if the sender’s power level is greater than or equal to the redact level, allow.
    if sender_level >= redact_level {
        info!("redaction allowed via power levels");
        return Ok(true);
    }

    // v1-v2, if the domain of the event_id of the event being redacted is the same as the
    // domain of the event_id of the m.room.redaction, allow.
    if room_redaction_event.event_id().borrow().server_name()
        == room_redaction_event.redacts().as_ref().and_then(|&id| id.borrow().server_name())
    {
        info!("redaction event allowed via room version 1 rules");
        return Ok(true);
    }

    // Otherwise, reject.
    Ok(false)
}

trait FetchStateExt<E: Event> {
    fn room_create_event(&self) -> std::result::Result<RoomCreateEvent<E>, String>;

    fn user_membership(&self, user_id: &UserId) -> std::result::Result<MembershipState, String>;

    fn room_power_levels_event(&self) -> Option<RoomPowerLevelsEvent<E>>;

    fn join_rule(&self) -> std::result::Result<JoinRule, String>;
}

impl<E, F> FetchStateExt<E> for F
where
    F: Fn(&StateEventType, &str) -> Option<E>,
    E: Event,
{
    fn room_create_event(&self) -> std::result::Result<RoomCreateEvent<E>, String> {
        self(&StateEventType::RoomCreate, "")
            .map(RoomCreateEvent::new)
            .ok_or_else(|| "no `m.room.create` event in current state".to_owned())
    }

    fn user_membership(&self, user_id: &UserId) -> std::result::Result<MembershipState, String> {
        self(&StateEventType::RoomMember, user_id.as_str()).map(RoomMemberEvent::new).membership()
    }

    fn room_power_levels_event(&self) -> Option<RoomPowerLevelsEvent<E>> {
        self(&StateEventType::RoomPowerLevels, "").map(RoomPowerLevelsEvent::new)
    }

    fn join_rule(&self) -> std::result::Result<JoinRule, String> {
        self(&StateEventType::RoomJoinRules, "")
            .map(RoomJoinRulesEvent::new)
            .ok_or_else(|| "no `m.room.join_rules` event in current state".to_owned())?
            .join_rule()
    }
}
