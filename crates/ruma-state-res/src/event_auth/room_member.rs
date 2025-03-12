use std::borrow::Borrow;

use js_int::int;
use ruma_common::{serde::Raw, UserId};
use ruma_events::{
    room::{
        join_rules::{JoinRule, RoomJoinRulesEventContent},
        member::{MembershipState, ThirdPartyInvite},
        power_levels::RoomPowerLevelsEventContent,
        third_party_invite::RoomThirdPartyInviteEventContent,
    },
    StateEventType,
};
use serde::Deserialize;
use serde_json::from_str as from_json_str;
use tracing::{debug, warn};

#[cfg(test)]
mod tests;

use super::{GetMembership, RoomMemberContentFields};
use crate::{
    events::{
        deserialize_power_levels_content_fields, deserialize_power_levels_content_invite,
        RoomCreateEvent,
    },
    Error, Event, Result, RoomVersion,
};

/// Check whether the given event passes the `m.room.roomber` authorization rules.
///
/// This assumes that `ruma_signatures::verify_event()` was called previously, as some authorization
/// rules depend on the signatures being valid on the event.
pub(super) fn check_room_member<E: Event>(
    room_member_event: impl Event,
    room_version: &RoomVersion,
    room_create_event: RoomCreateEvent<E>,
    fetch_state: impl Fn(&StateEventType, &str) -> Option<E>,
) -> Result<bool> {
    debug!("starting m.room.member check");

    // Since v1, if there is no state_key property, or no membership property in content,
    // reject.
    let state_key = match room_member_event.state_key() {
        Some(s) => s,
        None => {
            warn!("no statekey in member event");
            return Ok(false);
        }
    };
    let target_user =
        <&UserId>::try_from(state_key).map_err(|e| Error::InvalidPdu(format!("{e}")))?;

    let content: RoomMemberContentFields = from_json_str(room_member_event.content().get())?;

    let Some(target_membership) = content.membership.as_ref().and_then(|m| m.deserialize().ok())
    else {
        warn!("no valid membership field found for m.room.member event content");
        return Ok(false);
    };

    // These checks are done `in ruma_signatures::verify_event()`:
    //
    // Since v8, if content has a join_authorised_via_users_server property:
    //
    // - Since v8, if the event is not validly signed by the homeserver of the user ID denoted by
    //   the key, reject.

    match target_membership {
        // Since v1, if membership is join:
        MembershipState::Join => check_room_member_join(
            &room_member_event,
            target_user,
            content,
            room_version,
            room_create_event,
            fetch_state,
        ),
        // Since v1, if membership is invite:
        MembershipState::Invite => {
            check_room_member_invite(&room_member_event, target_user, fetch_state)
        }
        // Since v1, if membership is leave:
        MembershipState::Leave => {
            check_room_member_leave(&room_member_event, target_user, room_version, fetch_state)
        }
        // Since v1, if membership is ban:
        MembershipState::Ban => check_room_member_ban(&room_member_event, target_user, fetch_state),
        // Since v7, if membership is knock:
        MembershipState::Knock if room_version.allow_knocking => {
            check_room_member_knock(&room_member_event, target_user, room_version, fetch_state)
        }
        // Since v1, otherwise, the membership is unknown. Reject.
        _ => {
            warn!("unknown membership transition");
            Ok(false)
        }
    }
}

/// Check whether the given event passes the `m.room.member` authorization rules with a membership
/// of `join`.
fn check_room_member_join<E: Event>(
    room_member_event: &impl Event,
    target_user: &UserId,
    content: RoomMemberContentFields,
    room_version: &RoomVersion,
    room_create_event: RoomCreateEvent<E>,
    fetch_state: impl Fn(&StateEventType, &str) -> Option<E>,
) -> Result<bool> {
    let mut prev_events = room_member_event.prev_events();
    let prev_event_is_room_create_event = prev_events
        .next()
        .is_some_and(|event_id| event_id.borrow() == room_create_event.event_id().borrow());
    let prev_event_is_only_room_create_event =
        prev_event_is_room_create_event && prev_events.next().is_none();

    // v1-v10, if the only previous event is an m.room.create and the state_key is the
    // creator, allow.
    // Since v11, if the only previous event is an m.room.create and the state_key is the
    // sender of the m.room.create, allow.
    if prev_event_is_only_room_create_event {
        let creator = room_create_event.creator(room_version).map_err(Error::custom)?;

        if *target_user == *creator {
            return Ok(true);
        }
    }

    // Since v1, if the sender does not match state_key, reject.
    if room_member_event.sender() != target_user {
        warn!("can't make other user join");
        return Ok(false);
    }

    let current_room_member_event = fetch_state(&StateEventType::RoomMember, target_user.as_str());
    let current_membership = match &current_room_member_event {
        Some(event) => from_json_str::<GetMembership>(event.content().get())?.membership,
        None => MembershipState::Leave,
    };

    // Since v1, if the sender is banned, reject.
    if current_membership == MembershipState::Ban {
        warn!(target_user_membership_event_id = ?current_room_member_event.as_ref().map(|event| event.event_id()), "banned user can't join");
        return Ok(false);
    }

    let room_join_rules_event = fetch_state(&StateEventType::RoomJoinRules, "");
    let join_rule = match &room_join_rules_event {
        Some(event) => from_json_str::<RoomJoinRulesEventContent>(event.content().get())?.join_rule,
        None => JoinRule::Invite,
    };

    // v1-v6, if the join_rule is invite then allow if membership state is invite or
    // join.
    // Since v7, if the join_rule is invite or knock then allow if membership state is
    // invite or join.
    if (join_rule == JoinRule::Invite
        || room_version.allow_knocking && join_rule == JoinRule::Knock)
        && matches!(current_membership, MembershipState::Invite | MembershipState::Join)
    {
        return Ok(true);
    }

    // v8-v9, if the join_rule is restricted:
    // Since v10, if the join_rule is restricted or knock_restricted:
    if room_version.restricted_join_rules && matches!(join_rule, JoinRule::Restricted(_))
        || room_version.knock_restricted_join_rule
            && matches!(join_rule, JoinRule::KnockRestricted(_))
    {
        // Since v8, if membership state is join or invite, allow.
        if matches!(current_membership, MembershipState::Join | MembershipState::Invite) {
            return Ok(true);
        }

        // Since v8, if the join_authorised_via_users_server key in content is not a
        // user with sufficient permission to invite other users, reject.
        //
        // Otherwise, allow.
        let Some(authorized_via_user) =
            content.join_authorised_via_users_server.as_ref().and_then(|u| u.deserialize().ok())
        else {
            // The field is absent, we cannot authorize.
            return Ok(false);
        };

        // The member needs to be in the room to have any kind of permission.
        let authorized_via_user_room_member_event =
            fetch_state(&StateEventType::RoomMember, authorized_via_user.as_str());
        let authorized_via_user_is_joined = authorized_via_user_room_member_event
            .and_then(|event| from_json_str::<GetMembership>(event.content().get()).ok())
            .is_some_and(|content| content.membership == MembershipState::Join);
        if !authorized_via_user_is_joined {
            return Ok(false);
        }

        let room_power_levels_event = fetch_state(&StateEventType::RoomPowerLevels, "");

        let (authorized_via_user_level, invite_level) =
            if let Some(event) = &room_power_levels_event {
                // TODO Refactor all powerlevel parsing
                let invite =
                    deserialize_power_levels_content_invite(event.content().get(), room_version)?
                        .invite;

                let content =
                    deserialize_power_levels_content_fields(event.content().get(), room_version)?;
                let user_level = if let Some(user_level) = content.users.get(&authorized_via_user) {
                    *user_level
                } else {
                    content.users_default
                };

                (user_level, invite)
            } else {
                (int!(0), int!(0))
            };

        return Ok(authorized_via_user_level >= invite_level);
    }

    // Since v1, if the join_rule is public, allow.
    // Otherwise, reject.
    let allow = join_rule == JoinRule::Public;

    if !allow {
        warn!("Can't join a non-public room");
    }

    Ok(allow)
}

/// Check whether the given event passes the `m.room.member` authorization rules with a membership
/// of `invite`.
fn check_room_member_invite<E: Event>(
    room_member_event: &impl Event,
    target_user: &UserId,
    fetch_state: impl Fn(&StateEventType, &str) -> Option<E>,
) -> Result<bool> {
    #[derive(Deserialize)]
    struct GetThirdPartyInvite {
        third_party_invite: Option<Raw<ThirdPartyInvite>>,
    }

    let third_party_invite =
        from_json_str::<GetThirdPartyInvite>(room_member_event.content().get())?.third_party_invite;

    // Since v1, if content has a third_party_invite property:
    if let Some(raw_third_party_invite) = third_party_invite {
        return check_third_party_invite(
            room_member_event,
            raw_third_party_invite,
            target_user,
            fetch_state,
        );
    }

    let sender_room_member_event =
        fetch_state(&StateEventType::RoomMember, room_member_event.sender().as_str());
    let sender_membership = match &sender_room_member_event {
        Some(event) => from_json_str::<GetMembership>(event.content().get())?.membership,
        None => MembershipState::Leave,
    };

    // Since v1, if the sender’s current membership state is not join, reject.
    if sender_membership != MembershipState::Join {
        warn!(
            sender_membership_event_id = ?sender_room_member_event.as_ref().map(|event| event.event_id()),
            "can't invite user if sender not joined",
        );
        return Ok(false);
    }

    let current_target_user_room_member_event =
        fetch_state(&StateEventType::RoomMember, target_user.as_str());
    let current_target_user_membership = match &current_target_user_room_member_event {
        Some(event) => from_json_str::<GetMembership>(event.content().get())?.membership,
        None => MembershipState::Leave,
    };

    // Since v1, if target user’s current membership state is join or ban, reject.
    if matches!(current_target_user_membership, MembershipState::Join | MembershipState::Ban) {
        warn!(
            target_user_membership_event_id =
                ?current_target_user_room_member_event.as_ref().map(|event| event.event_id()),
            "can't invite user if user is currently joined or banned",
        );
        return Ok(false);
    }

    let room_power_levels_event = fetch_state(&StateEventType::RoomPowerLevels, "");
    let power_levels: RoomPowerLevelsEventContent = match &room_power_levels_event {
        Some(event) => from_json_str(event.content().get())?,
        None => RoomPowerLevelsEventContent::default(),
    };

    let sender_level =
        power_levels.users.get(room_member_event.sender()).unwrap_or(&power_levels.users_default);

    // Since v1, if the sender’s power level is greater than or equal to the invite
    // level, allow.
    //
    // Otherwise, reject.
    let allow = sender_level >= &power_levels.invite;

    if !allow {
        warn!(
            target_user_membership_event_id =
                ?current_target_user_room_member_event.as_ref().map(|event| event.event_id()),
            power_levels_event_id =
                ?room_power_levels_event.as_ref().map(|event| event.event_id()),
            "user does not have enough power to invite",
        );
    }

    Ok(allow)
}

/// Check whether the `third_party_invite` from the `m.room.member` event passes the authorization
/// rules.
fn check_third_party_invite<E: Event>(
    room_member_event: &impl Event,
    raw_third_party_invite: Raw<ThirdPartyInvite>,
    target_user: &UserId,
    fetch_state: impl Fn(&StateEventType, &str) -> Option<E>,
) -> Result<bool> {
    let current_target_user_room_member_event =
        fetch_state(&StateEventType::RoomMember, target_user.as_str());
    let current_target_user_membership = match &current_target_user_room_member_event {
        Some(event) => from_json_str::<GetMembership>(event.content().get())?.membership,
        None => MembershipState::Leave,
    };

    if current_target_user_membership == MembershipState::Ban {
        // Since v1, if target user is banned, reject.
        warn!(
            target_user_membership_event_id = ?current_target_user_room_member_event.as_ref().map(|event| event.event_id()),
            "can't invite banned user"
        );
        return Ok(false);
    }

    // Since v1, if content.third_party_invite does not have a signed property, reject.
    // Since v1, if signed does not have mxid and token properties, reject.
    let third_party_invite = raw_third_party_invite.deserialize()?;

    // Since v1, if mxid does not match state_key, reject.
    if target_user != third_party_invite.signed.mxid {
        warn!("mxid doesn't match state key");
        return Ok(false);
    }

    // Since v1, if there is no m.room.third_party_invite event in the current room state with
    // state_key matching token, reject.
    let Some(room_third_party_invite_event) =
        fetch_state(&StateEventType::RoomThirdPartyInvite, &third_party_invite.signed.token)
    else {
        warn!("no m.room.third_party_event in state matches the token");
        return Ok(false);
    };

    // Since v1, if sender does not match sender of the m.room.third_party_invite, reject.
    if room_member_event.sender() != room_third_party_invite_event.sender() {
        warn!("sender of m.room.third_party_invite doesn't match sender of m.room.member");
        return Ok(false);
    }

    let _room_third_party_invite_content = from_json_str::<RoomThirdPartyInviteEventContent>(
        room_third_party_invite_event.content().get(),
    )?;

    // Since v1, if any signature in signed matches any public key in the m.room.third_party_invite
    // event, allow.
    //
    // Otherwise, reject.
    //
    // FIXME: verify the signatures on `signed` with the public keys in `m.room.third_party_invite`.
    // For now let's accept the event.
    Ok(true)
}

/// Check whether the given event passes the `m.room.member` authorization rules with a membership
/// of `leave`.
fn check_room_member_leave<E: Event>(
    room_member_event: &impl Event,
    target_user: &UserId,
    room_version: &RoomVersion,
    fetch_state: impl Fn(&StateEventType, &str) -> Option<E>,
) -> Result<bool> {
    let sender_room_member_event =
        fetch_state(&StateEventType::RoomMember, room_member_event.sender().as_str());
    let sender_membership = match &sender_room_member_event {
        Some(event) => from_json_str::<GetMembership>(event.content().get())?.membership,
        None => MembershipState::Leave,
    };

    // v1-v6, if the sender matches state_key, allow if and only if that user’s current
    // membership state is invite or join.
    // Since v7, if the sender matches state_key, allow if and only if that user’s current
    // membership state is invite, join, or knock.
    if room_member_event.sender() == target_user {
        let allow = matches!(sender_membership, MembershipState::Join | MembershipState::Invite)
            || (room_version.allow_knocking && sender_membership == MembershipState::Knock);

        if !allow {
            warn!(
                target_user_membership_event_id = ?sender_room_member_event.as_ref().map(|event| event.event_id()),
                "can't leave if not joined, invited or knocked"
            );
        }

        return Ok(allow);
    }

    // Since v1, if the sender’s current membership state is not join, reject.
    if sender_membership != MembershipState::Join {
        warn!(
            sender_membership_event_id = ?sender_room_member_event.as_ref().map(|event| event.event_id()),
            "can't kick if sender not joined",
        );
        return Ok(false);
    }

    let current_target_user_room_member_event =
        fetch_state(&StateEventType::RoomMember, target_user.as_str());
    let current_target_user_membership = match &current_target_user_room_member_event {
        Some(event) => from_json_str::<GetMembership>(event.content().get())?.membership,
        None => MembershipState::Leave,
    };

    let room_power_levels_event = fetch_state(&StateEventType::RoomPowerLevels, "");
    let power_levels: RoomPowerLevelsEventContent = match &room_power_levels_event {
        Some(event) => from_json_str(event.content().get())?,
        None => RoomPowerLevelsEventContent::default(),
    };

    let sender_level =
        power_levels.users.get(room_member_event.sender()).unwrap_or(&power_levels.users_default);

    // Since v1, if the target user’s current membership state is ban, and the sender’s
    // power level is less than the ban level, reject.
    if current_target_user_membership == MembershipState::Ban && sender_level < &power_levels.ban {
        warn!(
            target_user_membership_event_id = ?current_target_user_room_member_event.as_ref().map(|event| event.event_id()),
            sender_membership_event_id = ?sender_room_member_event.as_ref().map(|event| event.event_id()),
            "can't kick if user is banned and sender can't unban",
        );
        return Ok(false);
    }

    let target_user_level =
        power_levels.users.get(target_user).unwrap_or(&power_levels.users_default);

    // Since v1, if the sender’s power level is greater than or equal to the kick level,
    // and the target user’s power level is less than the sender’s power level, allow.
    //
    // Otherwise, reject.
    let allow = sender_level >= &power_levels.kick && target_user_level < sender_level;

    if !allow {
        warn!(
            target_user_membership_event_id = ?current_target_user_room_member_event.as_ref().map(|event| event.event_id()),
            power_levels_event_id = ?room_power_levels_event.as_ref().map(|event| event.event_id()),
            "sender does not have enough power to kick target user",
        );
    }

    Ok(allow)
}

/// Check whether the given event passes the `m.room.member` authorization rules with a membership
/// of `ban`.
fn check_room_member_ban<E: Event>(
    room_member_event: &impl Event,
    target_user: &UserId,
    fetch_state: impl Fn(&StateEventType, &str) -> Option<E>,
) -> Result<bool> {
    let sender_room_member_event =
        fetch_state(&StateEventType::RoomMember, room_member_event.sender().as_str());
    let sender_membership = match &sender_room_member_event {
        Some(event) => from_json_str::<GetMembership>(event.content().get())?.membership,
        None => MembershipState::Leave,
    };

    // Since v1, if the sender’s current membership state is not join, reject.
    if sender_membership != MembershipState::Join {
        warn!(
            sender_membership_event_id = ?sender_room_member_event.as_ref().map(|event| event.event_id()),
            "can't ban user if sender is not joined"
        );
        return Ok(false);
    }
    let room_power_levels_event = fetch_state(&StateEventType::RoomPowerLevels, "");
    let power_levels: RoomPowerLevelsEventContent = match &room_power_levels_event {
        Some(event) => from_json_str(event.content().get())?,
        None => RoomPowerLevelsEventContent::default(),
    };

    let sender_level =
        power_levels.users.get(room_member_event.sender()).unwrap_or(&power_levels.users_default);
    let target_user_level =
        power_levels.users.get(target_user).unwrap_or(&power_levels.users_default);

    // If the sender’s power level is greater than or equal to the ban level, and the
    // target user’s power level is less than the sender’s power level, allow.
    //
    // Otherwise, reject.
    let allow = sender_level >= &power_levels.ban && target_user_level < sender_level;

    if !allow {
        warn!(
            sender_membership_event_id = ?sender_room_member_event.as_ref().map(|event| event.event_id()),
            power_levels_event_id = ?room_power_levels_event.as_ref().map(|event| event.event_id()),
            "sender does not have enough power to ban target user",
        );
    }

    Ok(allow)
}

/// Check whether the given event passes the `m.room.member` authorization rules with a membership
/// of `knock`.
fn check_room_member_knock<E: Event>(
    room_member_event: &impl Event,
    target_user: &UserId,
    room_version: &RoomVersion,
    fetch_state: impl Fn(&StateEventType, &str) -> Option<E>,
) -> Result<bool> {
    let room_join_rules_event = fetch_state(&StateEventType::RoomJoinRules, "");
    let join_rule = match &room_join_rules_event {
        Some(event) => from_json_str::<RoomJoinRulesEventContent>(event.content().get())?.join_rule,
        None => JoinRule::Invite,
    };

    // v7-v9, if the join_rule is anything other than knock, reject.
    // Since v10, if the join_rule is anything other than knock or knock_restricted,
    // reject.
    if join_rule != JoinRule::Knock
        && (room_version.knock_restricted_join_rule
            && !matches!(join_rule, JoinRule::KnockRestricted(_)))
    {
        warn!("join rule is not set to knock or knock_restricted, knocking is not allowed");
        return Ok(false);
    }

    // Since v7, if sender does not match state_key, reject.
    if room_member_event.sender() != target_user {
        warn!(
            sender = ?room_member_event.sender(),
            ?target_user,
            "can't make another user knock, sender did not match target"
        );
        return Ok(false);
    }

    let sender_room_member_event =
        fetch_state(&StateEventType::RoomMember, room_member_event.sender().as_str());
    let sender_membership = match &sender_room_member_event {
        Some(event) => from_json_str::<GetMembership>(event.content().get())?.membership,
        None => MembershipState::Leave,
    };

    // Since v7, if the sender’s current membership is not ban, invite, or join, allow.
    // Otherwise, reject.
    let allow = !matches!(
        sender_membership,
        MembershipState::Ban | MembershipState::Invite | MembershipState::Join
    );

    if !allow {
        warn!(
            target_user_membership_event_id = ?sender_room_member_event.as_ref().map(|event| event.event_id()),
            "cannot knock while current membership state is ban, invite or join",
        );
    }

    Ok(allow)
}
