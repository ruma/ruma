use std::{
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet, HashSet},
};

use js_int::Int;
use ruma_common::{room_version_rules::AuthorizationRules, EventId, UserId};
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
        RoomThirdPartyInviteEvent,
    },
    Event, StateEventType, TimelineEventType,
};

// TODO: We need methods for all checks performed on receipt of a PDU, plus the following that are
// not listed:
//
// - check that the event respects the size limits,
//
// References:
// - https://spec.matrix.org/latest/server-server-api/#checks-performed-on-receipt-of-a-pdu
// - https://spec.matrix.org/latest/client-server-api/#size-limits
// - https://github.com/element-hq/synapse/blob/9c5d08fff8d66a7cc0e2ecfeeb783f933a778c2f/synapse/event_auth.py
// - https://github.com/matrix-org/matrix-spec/issues/365

/// Get the list of [relevant auth events] required to authorize the event of the given type.
///
/// Returns a list of `(event_type, state_key)` tuples.
///
/// # Errors
///
/// Returns an `Err(_)` if a field could not be deserialized because `content` does not respect the
/// expected format for the `event_type`.
///
/// [relevant auth events]: https://spec.matrix.org/latest/server-server-api/#auth-events-selection
pub fn auth_types_for_event(
    event_type: &TimelineEventType,
    sender: &UserId,
    state_key: Option<&str>,
    content: &RawJsonValue,
    rules: &AuthorizationRules,
) -> Result<Vec<(StateEventType, String)>, String> {
    // The `auth_events` for the `m.room.create` event in a room is empty.
    if event_type == &TimelineEventType::RoomCreate {
        return Ok(vec![]);
    }

    // For other events, it should be the following subset of the room state:
    //
    // - The `m.room.create` event.
    // - The current `m.room.power_levels` event, if any.
    // - The sender’s current `m.room.member` event, if any.
    let mut auth_types = vec![
        (StateEventType::RoomPowerLevels, "".to_owned()),
        (StateEventType::RoomMember, sender.to_string()),
        (StateEventType::RoomCreate, "".to_owned()),
    ];

    // If type is `m.room.member`:
    if event_type == &TimelineEventType::RoomMember {
        // The target’s current `m.room.member` event, if any.
        let Some(state_key) = state_key else {
            return Err("missing `state_key` field for `m.room.member` event".to_owned());
        };
        let key = (StateEventType::RoomMember, state_key.to_owned());
        if !auth_types.contains(&key) {
            auth_types.push(key);
        }

        let content = RoomMemberEventContent::new(content);
        let membership = content.membership()?;

        // If `membership` is `join`, `invite` or `knock`, the current `m.room.join_rules` event, if
        // any.
        if matches!(
            membership,
            MembershipState::Join | MembershipState::Invite | MembershipState::Knock
        ) {
            let key = (StateEventType::RoomJoinRules, "".to_owned());
            if !auth_types.contains(&key) {
                auth_types.push(key);
            }
        }

        // If `membership` is `invite` and `content` contains a `third_party_invite` property, the
        // current `m.room.third_party_invite` event with `state_key` matching
        // `content.third_party_invite.signed.token`, if any.
        if membership == MembershipState::Invite {
            let third_party_invite = content.third_party_invite()?;

            if let Some(third_party_invite) = third_party_invite {
                let token = third_party_invite.token()?.to_owned();
                let key = (StateEventType::RoomThirdPartyInvite, token);
                if !auth_types.contains(&key) {
                    auth_types.push(key);
                }
            }
        }

        // If `content.join_authorised_via_users_server` is present, and the room version supports
        // restricted rooms, then the `m.room.member` event with `state_key` matching
        // `content.join_authorised_via_users_server`.
        //
        // Note: And the membership is join (https://github.com/matrix-org/matrix-spec/pull/2100)
        if membership == MembershipState::Join && rules.restricted_join_rule {
            let join_authorised_via_users_server = content.join_authorised_via_users_server()?;
            if let Some(user_id) = join_authorised_via_users_server {
                let key = (StateEventType::RoomMember, user_id.to_string());
                if !auth_types.contains(&key) {
                    auth_types.push(key);
                }
            }
        }
    }

    Ok(auth_types)
}

/// Check whether the incoming event passes the state-independent [authorization rules] for the
/// given room version rules.
///
/// The state-independent rules are the first few authorization rules that check an incoming
/// `m.room.create` event (which cannot have `auth_events`), and the list of `auth_events` of other
/// events.
///
/// This method only needs to be called once, when the event is received.
///
/// # Errors
///
/// If the check fails, this returns an `Err(_)` with a description of the check that failed.
///
/// [authorization rules]: https://spec.matrix.org/latest/server-server-api/#authorization-rules
#[instrument(skip_all, fields(event_id = incoming_event.event_id().borrow().as_str()))]
pub fn check_state_independent_auth_rules<E: Event>(
    rules: &AuthorizationRules,
    incoming_event: impl Event,
    fetch_event: impl Fn(&EventId) -> Option<E>,
) -> Result<(), String> {
    debug!("starting state-independent auth check");

    // Since v1, if type is m.room.create:
    if *incoming_event.event_type() == TimelineEventType::RoomCreate {
        let room_create_event = RoomCreateEvent::new(incoming_event);

        return check_room_create(room_create_event, rules);
    }

    let expected_auth_types = auth_types_for_event(
        incoming_event.event_type(),
        incoming_event.sender(),
        incoming_event.state_key(),
        incoming_event.content(),
        rules,
    )?
    .into_iter()
    .map(|(event_type, state_key)| (TimelineEventType::from(event_type), state_key))
    .collect::<HashSet<_>>();

    let room_id = incoming_event.room_id();
    let mut seen_auth_types: HashSet<(TimelineEventType, String)> =
        HashSet::with_capacity(expected_auth_types.len());

    // Since v1, considering auth_events:
    for auth_event_id in incoming_event.auth_events() {
        let event_id = auth_event_id.borrow();

        let Some(auth_event) = fetch_event(event_id) else {
            return Err(format!("failed to find auth event {event_id}"));
        };

        // The auth event must be in the same room as the incoming event.
        if auth_event.room_id() != room_id {
            return Err(format!("auth event {event_id} not in the same room"));
        }

        let event_type = auth_event.event_type();
        let state_key = auth_event
            .state_key()
            .ok_or_else(|| format!("auth event {event_id} has no `state_key`"))?;
        let key = (event_type.clone(), state_key.to_owned());

        // Since v1, if there are duplicate entries for a given type and state_key pair, reject.
        if seen_auth_types.contains(&key) {
            return Err(format!(
                "duplicate auth event {event_id} for ({event_type}, {state_key}) pair"
            ));
        }

        // Since v1, if there are entries whose type and state_key don’t match those specified by
        // the auth events selection algorithm described in the server specification, reject.
        if !expected_auth_types.contains(&key) {
            return Err(format!(
                "unexpected auth event {event_id} with ({event_type}, {state_key}) pair"
            ));
        };

        // Since v1, if there are entries which were themselves rejected under the checks performed
        // on receipt of a PDU, reject.
        if auth_event.rejected() {
            return Err(format!("rejected auth event {event_id}"));
        }

        seen_auth_types.insert(key);
    }

    // Since v1, if there is no m.room.create event among the entries, reject.
    if !seen_auth_types.iter().any(|(event_type, _)| *event_type == TimelineEventType::RoomCreate) {
        return Err("no `m.room.create` event in auth events".to_owned());
    }

    Ok(())
}

/// Check whether the incoming event passes the state-dependent [authorization rules] for the given
/// room version rules.
///
/// The state-dependent rules are all the remaining rules not checked by
/// [`check_state_independent_auth_rules()`].
///
/// This method should be called several times for an event, to perform the [checks on receipt of a
/// PDU].
///
/// The `fetch_state` closure should gather state from a state snapshot. We need to know if the
/// event passes auth against some state not a recursive collection of auth_events fields.
///
/// This assumes that `ruma_signatures::verify_event()` was called previously, as some authorization
/// rules depend on the signatures being valid on the event.
///
/// # Errors
///
/// If the check fails, this returns an `Err(_)` with a description of the check that failed.
///
/// [authorization rules]: https://spec.matrix.org/latest/server-server-api/#authorization-rules
/// [checks on receipt of a PDU]: https://spec.matrix.org/latest/server-server-api/#checks-performed-on-receipt-of-a-pdu
#[instrument(skip_all, fields(event_id = incoming_event.event_id().borrow().as_str()))]
pub fn check_state_dependent_auth_rules<E: Event>(
    rules: &AuthorizationRules,
    incoming_event: impl Event,
    fetch_state: impl Fn(&StateEventType, &str) -> Option<E>,
) -> Result<(), String> {
    debug!("starting state-dependent auth check");

    // There are no state-dependent auth rules for create events.
    if *incoming_event.event_type() == TimelineEventType::RoomCreate {
        debug!("allowing `m.room.create` event");
        return Ok(());
    }

    let room_create_event = fetch_state.room_create_event()?;

    // Since v1, if the create event content has the field m.federate set to false and the sender
    // domain of the event does not match the sender domain of the create event, reject.
    let federate = room_create_event.federate()?;
    if !federate
        && room_create_event.sender().server_name() != incoming_event.sender().server_name()
    {
        return Err("\
            room is not federated and event's sender domain \
            does not match `m.room.create` event's sender domain"
            .to_owned());
    }

    let sender = incoming_event.sender();

    // v1-v5, if type is m.room.aliases:
    if rules.special_case_room_aliases
        && *incoming_event.event_type() == TimelineEventType::RoomAliases
    {
        debug!("starting m.room.aliases check");
        // v1-v5, if event has no state_key, reject.
        //
        // v1-v5, if sender's domain doesn't match state_key, reject.
        if incoming_event.state_key() != Some(sender.server_name().as_str()) {
            return Err("\
                server name of the `state_key` of `m.room.aliases` event \
                does not match the server name of the sender"
                .to_owned());
        }

        // Otherwise, allow.
        info!("`m.room.aliases` event was allowed");
        return Ok(());
    }

    // Since v1, if type is m.room.member:
    if *incoming_event.event_type() == TimelineEventType::RoomMember {
        let room_member_event = RoomMemberEvent::new(incoming_event);
        return check_room_member(room_member_event, rules, room_create_event, fetch_state);
    }

    // Since v1, if the sender's current membership state is not join, reject.
    let sender_membership = fetch_state.user_membership(sender)?;

    if sender_membership != MembershipState::Join {
        return Err("sender's membership is not `join`".to_owned());
    }

    let creator = room_create_event.creator(rules)?;
    let current_room_power_levels_event = fetch_state.room_power_levels_event();

    let sender_power_level =
        current_room_power_levels_event.user_power_level(sender, &creator, rules)?;

    // Since v1, if type is m.room.third_party_invite:
    if *incoming_event.event_type() == TimelineEventType::RoomThirdPartyInvite {
        // Since v1, allow if and only if sender's current power level is greater than
        // or equal to the invite level.
        let invite_power_level = current_room_power_levels_event
            .get_as_int_or_default(RoomPowerLevelsIntField::Invite, rules)?;

        if sender_power_level < invite_power_level {
            return Err("sender does not have enough power to send invites in this room".to_owned());
        }

        info!("`m.room.third_party_invite` event was allowed");
        return Ok(());
    }

    // Since v1, if the event type's required power level is greater than the sender's power level,
    // reject.
    let event_type_power_level = current_room_power_levels_event.event_power_level(
        incoming_event.event_type(),
        incoming_event.state_key(),
        rules,
    )?;
    if sender_power_level < event_type_power_level {
        return Err(format!(
            "sender does not have enough power to send event of type `{}`",
            incoming_event.event_type()
        ));
    }

    // Since v1, if the event has a state_key that starts with an @ and does not match the sender,
    // reject.
    if incoming_event.state_key().is_some_and(|k| k.starts_with('@'))
        && incoming_event.state_key() != Some(incoming_event.sender().as_str())
    {
        return Err(
            "sender cannot send event with `state_key` matching another user's ID".to_owned()
        );
    }

    // If type is m.room.power_levels
    if *incoming_event.event_type() == TimelineEventType::RoomPowerLevels {
        let room_power_levels_event = RoomPowerLevelsEvent::new(incoming_event);
        return check_room_power_levels(
            room_power_levels_event,
            current_room_power_levels_event,
            rules,
            sender_power_level,
        );
    }

    // v1-v2, if type is m.room.redaction:
    if rules.special_case_room_redaction
        && *incoming_event.event_type() == TimelineEventType::RoomRedaction
    {
        return check_room_redaction(
            incoming_event,
            current_room_power_levels_event,
            rules,
            sender_power_level,
        );
    }

    // Otherwise, allow.
    info!("allowing event passed all checks");
    Ok(())
}

/// Check whether the given event passes the `m.room.create` authorization rules.
fn check_room_create(
    room_create_event: RoomCreateEvent<impl Event>,
    rules: &AuthorizationRules,
) -> Result<(), String> {
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
    // This check is assumed to be done before calling auth_check because we have an
    // AuthorizationRules, which means that we recognized the version.

    // v1-v10, if content has no creator field, reject.
    if !rules.use_room_create_sender && !room_create_event.has_creator()? {
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
    rules: &AuthorizationRules,
    sender_power_level: Int,
) -> Result<(), String> {
    debug!("starting m.room.power_levels check");

    // Since v10, if any of the properties users_default, events_default, state_default, ban,
    // redact, kick, or invite in content are present and not an integer, reject.
    let new_int_fields = room_power_levels_event.int_fields_map(rules)?;

    // Since v10, if either of the properties events or notifications in content are present and not
    // a dictionary with values that are integers, reject.
    let new_events = room_power_levels_event.events(rules)?;
    let new_notifications = room_power_levels_event.notifications(rules)?;

    // v1-v9, If the users property in content is not an object with keys that are valid user IDs
    // with values that are integers (or a string that is an integer), reject.
    // Since v10, if the users property in content is not an object with keys that are valid user
    // IDs with values that are integers, reject.
    let new_users = room_power_levels_event.users(rules)?;

    debug!("validation of power event finished");

    // Since v1, if there is no previous m.room.power_levels event in the room, allow.
    let Some(current_room_power_levels_event) = current_room_power_levels_event else {
        info!("initial m.room.power_levels event allowed");
        return Ok(());
    };

    // Since v1, for the properties users_default, events_default, state_default, ban, redact, kick,
    // invite check if they were added, changed or removed. For each found alteration:
    for field in RoomPowerLevelsIntField::ALL {
        let current_power_level = current_room_power_levels_event.get_as_int(*field, rules)?;
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
    let current_events = current_room_power_levels_event.events(rules)?;
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
    if rules.limit_notifications_power_levels {
        let current_notifications = current_room_power_levels_event.notifications(rules)?;
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
    let current_users = current_room_power_levels_event.users(rules)?;
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
) -> Result<(), String> {
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
    rules: &AuthorizationRules,
    sender_level: Int,
) -> Result<(), String> {
    let redact_level = current_room_power_levels_event
        .get_as_int_or_default(RoomPowerLevelsIntField::Redact, rules)?;

    // v1-v2, if the sender’s power level is greater than or equal to the redact level, allow.
    if sender_level >= redact_level {
        info!("`m.room.redaction` event allowed via power levels");
        return Ok(());
    }

    // v1-v2, if the domain of the event_id of the event being redacted is the same as the
    // domain of the event_id of the m.room.redaction, allow.
    if room_redaction_event.event_id().borrow().server_name()
        == room_redaction_event.redacts().as_ref().and_then(|&id| id.borrow().server_name())
    {
        info!("`m.room.redaction` event allowed via room version 1 rules");
        return Ok(());
    }

    // Otherwise, reject.
    Err("`m.room.redaction` event did not pass any of the allow rules".to_owned())
}

trait FetchStateExt<E: Event> {
    fn room_create_event(&self) -> Result<RoomCreateEvent<E>, String>;

    fn user_membership(&self, user_id: &UserId) -> Result<MembershipState, String>;

    fn room_power_levels_event(&self) -> Option<RoomPowerLevelsEvent<E>>;

    fn join_rule(&self) -> Result<JoinRule, String>;

    fn room_third_party_invite_event(&self, token: &str) -> Option<RoomThirdPartyInviteEvent<E>>;
}

impl<E, F> FetchStateExt<E> for F
where
    F: Fn(&StateEventType, &str) -> Option<E>,
    E: Event,
{
    fn room_create_event(&self) -> Result<RoomCreateEvent<E>, String> {
        self(&StateEventType::RoomCreate, "")
            .map(RoomCreateEvent::new)
            .ok_or_else(|| "no `m.room.create` event in current state".to_owned())
    }

    fn user_membership(&self, user_id: &UserId) -> Result<MembershipState, String> {
        self(&StateEventType::RoomMember, user_id.as_str()).map(RoomMemberEvent::new).membership()
    }

    fn room_power_levels_event(&self) -> Option<RoomPowerLevelsEvent<E>> {
        self(&StateEventType::RoomPowerLevels, "").map(RoomPowerLevelsEvent::new)
    }

    fn join_rule(&self) -> Result<JoinRule, String> {
        self(&StateEventType::RoomJoinRules, "")
            .map(RoomJoinRulesEvent::new)
            .ok_or_else(|| "no `m.room.join_rules` event in current state".to_owned())?
            .join_rule()
    }

    fn room_third_party_invite_event(&self, token: &str) -> Option<RoomThirdPartyInviteEvent<E>> {
        self(&StateEventType::RoomThirdPartyInvite, token).map(RoomThirdPartyInviteEvent::new)
    }
}
