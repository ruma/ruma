use std::borrow::Borrow;

use js_int::int;
use ruma_common::UserId;
use ruma_events::{
    room::{
        join_rules::{JoinRule, RoomJoinRulesEventContent},
        member::MembershipState,
        power_levels::RoomPowerLevelsEventContent,
        third_party_invite::RoomThirdPartyInviteEventContent,
    },
    StateEventType,
};
use serde_json::from_str as from_json_str;
use tracing::debug;

#[cfg(test)]
mod tests;

use super::FetchStateExt;
use crate::{
    events::{
        deserialize_power_levels_content_fields, deserialize_power_levels_content_invite,
        member::ThirdPartyInvite, RoomCreateEvent, RoomMemberEvent,
    },
    Event, RoomVersion,
};

/// Check whether the given event passes the `m.room.roomber` authorization rules.
///
/// This assumes that `ruma_signatures::verify_event()` was called previously, as some authorization
/// rules depend on the signatures being valid on the event.
pub(super) fn check_room_member<E: Event>(
    room_member_event: RoomMemberEvent<impl Event>,
    room_version: &RoomVersion,
    room_create_event: RoomCreateEvent<E>,
    fetch_state: impl Fn(&StateEventType, &str) -> Option<E>,
) -> Result<(), String> {
    debug!("starting m.room.member check");

    // Since v1, if there is no state_key property, or no membership property in content,
    // reject.
    let Some(state_key) = room_member_event.state_key() else {
        return Err("missing `state_key` field in `m.room.member` event".to_owned());
    };
    let target_user = <&UserId>::try_from(state_key)
        .map_err(|e| format!("invalid `state_key` field in `m.room.member` event: {e}"))?;

    let target_membership = room_member_event.membership()?;

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
        _ => Err("unknown membership".to_owned()),
    }
}

/// Check whether the given event passes the `m.room.member` authorization rules with a membership
/// of `join`.
fn check_room_member_join<E: Event>(
    room_member_event: &RoomMemberEvent<impl Event>,
    target_user: &UserId,
    room_version: &RoomVersion,
    room_create_event: RoomCreateEvent<E>,
    fetch_state: impl Fn(&StateEventType, &str) -> Option<E>,
) -> Result<(), String> {
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
        let creator = room_create_event.creator(room_version)?;

        if *target_user == *creator {
            return Ok(());
        }
    }

    // Since v1, if the sender does not match state_key, reject.
    if room_member_event.sender() != target_user {
        return Err("sender of join event must match target user".to_owned());
    }

    let current_membership = fetch_state.user_membership(target_user)?;

    // Since v1, if the sender is banned, reject.
    if current_membership == MembershipState::Ban {
        return Err("banned user cannot join room".to_owned());
    }

    let room_join_rules_event = fetch_state(&StateEventType::RoomJoinRules, "");
    let join_rule = match &room_join_rules_event {
        Some(event) => {
            from_json_str::<RoomJoinRulesEventContent>(event.content().get())
                .map_err(|error| error.to_string())?
                .join_rule
        }
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
        return Ok(());
    }

    // v8-v9, if the join_rule is restricted:
    // Since v10, if the join_rule is restricted or knock_restricted:
    if room_version.restricted_join_rules && matches!(join_rule, JoinRule::Restricted(_))
        || room_version.knock_restricted_join_rule
            && matches!(join_rule, JoinRule::KnockRestricted(_))
    {
        // Since v8, if membership state is join or invite, allow.
        if matches!(current_membership, MembershipState::Join | MembershipState::Invite) {
            return Ok(());
        }

        // Since v8, if the join_authorised_via_users_server key in content is not a
        // user with sufficient permission to invite other users, reject.
        //
        // Otherwise, allow.
        let Some(authorized_via_user) = room_member_event.join_authorised_via_users_server()?
        else {
            // The field is absent, we cannot authorize.
            return Err(
                "cannot join restricted room without `join_authorised_via_users_server` field \
                 if not invited"
                    .to_owned(),
            );
        };

        // The member needs to be in the room to have any kind of permission.
        let authorized_via_user_membership = fetch_state.user_membership(&authorized_via_user)?;
        if authorized_via_user_membership != MembershipState::Join {
            return Err("`join_authorised_via_users_server` is not joined".to_owned());
        }

        let room_power_levels_event = fetch_state(&StateEventType::RoomPowerLevels, "");

        let (authorized_via_user_level, invite_level) =
            if let Some(event) = &room_power_levels_event {
                // TODO Refactor all powerlevel parsing
                let invite =
                    deserialize_power_levels_content_invite(event.content().get(), room_version)
                        .map_err(|error| error.to_string())?
                        .invite;

                let content =
                    deserialize_power_levels_content_fields(event.content().get(), room_version)
                        .map_err(|error| error.to_string())?;
                let user_level = if let Some(user_level) = content.users.get(&authorized_via_user) {
                    *user_level
                } else {
                    content.users_default
                };

                (user_level, invite)
            } else {
                (int!(0), int!(0))
            };

        return if authorized_via_user_level >= invite_level {
            Ok(())
        } else {
            Err("`join_authorised_via_users_server` does not have enough power".to_owned())
        };
    }

    // Since v1, if the join_rule is public, allow.
    // Otherwise, reject.
    if join_rule == JoinRule::Public {
        Ok(())
    } else {
        Err("cannot join a room that is not `public`".to_owned())
    }
}

/// Check whether the given event passes the `m.room.member` authorization rules with a membership
/// of `invite`.
fn check_room_member_invite<E: Event>(
    room_member_event: &RoomMemberEvent<impl Event>,
    target_user: &UserId,
    fetch_state: impl Fn(&StateEventType, &str) -> Option<E>,
) -> Result<(), String> {
    let third_party_invite = room_member_event.third_party_invite()?;

    // Since v1, if content has a third_party_invite property:
    if let Some(third_party_invite) = third_party_invite {
        return check_third_party_invite(
            room_member_event,
            third_party_invite,
            target_user,
            fetch_state,
        );
    }

    let sender_membership = fetch_state.user_membership(room_member_event.sender())?;

    // Since v1, if the sender’s current membership state is not join, reject.
    if sender_membership != MembershipState::Join {
        return Err("cannot invite user if sender is not joined".to_owned());
    }

    let current_target_user_membership = fetch_state.user_membership(target_user)?;

    // Since v1, if target user’s current membership state is join or ban, reject.
    if matches!(current_target_user_membership, MembershipState::Join | MembershipState::Ban) {
        return Err("cannot invite user that is joined or banned".to_owned());
    }

    let room_power_levels_event = fetch_state(&StateEventType::RoomPowerLevels, "");
    let power_levels: RoomPowerLevelsEventContent = match &room_power_levels_event {
        Some(event) => from_json_str(event.content().get()).map_err(|error| error.to_string())?,
        None => RoomPowerLevelsEventContent::default(),
    };

    let sender_level =
        power_levels.users.get(room_member_event.sender()).unwrap_or(&power_levels.users_default);

    // Since v1, if the sender’s power level is greater than or equal to the invite
    // level, allow.
    //
    // Otherwise, reject.
    if sender_level >= &power_levels.invite {
        Ok(())
    } else {
        Err("sender does not have enough power to invite".to_owned())
    }
}

/// Check whether the `third_party_invite` from the `m.room.member` event passes the authorization
/// rules.
fn check_third_party_invite<E: Event>(
    room_member_event: &RoomMemberEvent<impl Event>,
    third_party_invite: ThirdPartyInvite,
    target_user: &UserId,
    fetch_state: impl Fn(&StateEventType, &str) -> Option<E>,
) -> Result<(), String> {
    let current_target_user_membership = fetch_state.user_membership(target_user)?;

    // Since v1, if target user is banned, reject.
    if current_target_user_membership == MembershipState::Ban {
        return Err("cannot invite user that is banned".to_owned());
    }

    // Since v1, if content.third_party_invite does not have a signed property, reject.
    // Since v1, if signed does not have mxid and token properties, reject.
    let third_party_invite_token = third_party_invite.token()?;
    let third_party_invite_mxid = third_party_invite.mxid()?;

    // Since v1, if mxid does not match state_key, reject.
    if target_user != third_party_invite_mxid {
        return Err("third-party invite mxid does not match target user".to_owned());
    }

    // Since v1, if there is no m.room.third_party_invite event in the current room state with
    // state_key matching token, reject.
    let Some(room_third_party_invite_event) =
        fetch_state(&StateEventType::RoomThirdPartyInvite, third_party_invite_token)
    else {
        return Err("no `m.room.third_party_invite` in room state matches the token".to_owned());
    };

    // Since v1, if sender does not match sender of the m.room.third_party_invite, reject.
    if room_member_event.sender() != room_third_party_invite_event.sender() {
        return Err(
            "sender of `m.room.third_party_invite` does not match sender of `m.room.member`"
                .to_owned(),
        );
    }

    let _room_third_party_invite_content = from_json_str::<RoomThirdPartyInviteEventContent>(
        room_third_party_invite_event.content().get(),
    )
    .map_err(|error| error.to_string())?;

    // Since v1, if any signature in signed matches any public key in the m.room.third_party_invite
    // event, allow.
    //
    // Otherwise, reject.
    //
    // FIXME: verify the signatures on `signed` with the public keys in `m.room.third_party_invite`.
    // For now let's accept the event.
    Ok(())
}

/// Check whether the given event passes the `m.room.member` authorization rules with a membership
/// of `leave`.
fn check_room_member_leave<E: Event>(
    room_member_event: &RoomMemberEvent<impl Event>,
    target_user: &UserId,
    room_version: &RoomVersion,
    fetch_state: impl Fn(&StateEventType, &str) -> Option<E>,
) -> Result<(), String> {
    let sender_membership = fetch_state.user_membership(room_member_event.sender())?;

    // v1-v6, if the sender matches state_key, allow if and only if that user’s current
    // membership state is invite or join.
    // Since v7, if the sender matches state_key, allow if and only if that user’s current
    // membership state is invite, join, or knock.
    if room_member_event.sender() == target_user {
        let membership_is_invite_or_join =
            matches!(sender_membership, MembershipState::Join | MembershipState::Invite);
        let membership_is_knock =
            room_version.allow_knocking && sender_membership == MembershipState::Knock;

        return if membership_is_invite_or_join || membership_is_knock {
            Ok(())
        } else {
            Err("cannot leave if not joined, invited or knocked".to_owned())
        };
    }

    // Since v1, if the sender’s current membership state is not join, reject.
    if sender_membership != MembershipState::Join {
        return Err("cannot kick if sender is not joined".to_owned());
    }

    let current_target_user_membership = fetch_state.user_membership(target_user)?;

    let room_power_levels_event = fetch_state(&StateEventType::RoomPowerLevels, "");
    let power_levels: RoomPowerLevelsEventContent = match &room_power_levels_event {
        Some(event) => from_json_str(event.content().get()).map_err(|error| error.to_string())?,
        None => RoomPowerLevelsEventContent::default(),
    };

    let sender_level =
        power_levels.users.get(room_member_event.sender()).unwrap_or(&power_levels.users_default);

    // Since v1, if the target user’s current membership state is ban, and the sender’s
    // power level is less than the ban level, reject.
    if current_target_user_membership == MembershipState::Ban && sender_level < &power_levels.ban {
        return Err("sender does not have enough power to unban".to_owned());
    }

    let target_user_level =
        power_levels.users.get(target_user).unwrap_or(&power_levels.users_default);

    // Since v1, if the sender’s power level is greater than or equal to the kick level,
    // and the target user’s power level is less than the sender’s power level, allow.
    //
    // Otherwise, reject.
    if sender_level >= &power_levels.kick && target_user_level < sender_level {
        Ok(())
    } else {
        Err("sender does not have enough power to kick target user".to_owned())
    }
}

/// Check whether the given event passes the `m.room.member` authorization rules with a membership
/// of `ban`.
fn check_room_member_ban<E: Event>(
    room_member_event: &RoomMemberEvent<impl Event>,
    target_user: &UserId,
    fetch_state: impl Fn(&StateEventType, &str) -> Option<E>,
) -> Result<(), String> {
    let sender_membership = fetch_state.user_membership(room_member_event.sender())?;

    // Since v1, if the sender’s current membership state is not join, reject.
    if sender_membership != MembershipState::Join {
        return Err("cannot ban if sender is not joined".to_owned());
    }

    let room_power_levels_event = fetch_state(&StateEventType::RoomPowerLevels, "");
    let power_levels: RoomPowerLevelsEventContent = match &room_power_levels_event {
        Some(event) => from_json_str(event.content().get()).map_err(|error| error.to_string())?,
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
    if sender_level >= &power_levels.ban && target_user_level < sender_level {
        Ok(())
    } else {
        Err("sender does not have enough power to ban target user".to_owned())
    }
}

/// Check whether the given event passes the `m.room.member` authorization rules with a membership
/// of `knock`.
fn check_room_member_knock<E: Event>(
    room_member_event: &RoomMemberEvent<impl Event>,
    target_user: &UserId,
    room_version: &RoomVersion,
    fetch_state: impl Fn(&StateEventType, &str) -> Option<E>,
) -> Result<(), String> {
    let room_join_rules_event = fetch_state(&StateEventType::RoomJoinRules, "");
    let join_rule = match &room_join_rules_event {
        Some(event) => {
            from_json_str::<RoomJoinRulesEventContent>(event.content().get())
                .map_err(|error| error.to_string())?
                .join_rule
        }
        None => JoinRule::Invite,
    };

    // v7-v9, if the join_rule is anything other than knock, reject.
    // Since v10, if the join_rule is anything other than knock or knock_restricted,
    // reject.
    if join_rule != JoinRule::Knock
        && (room_version.knock_restricted_join_rule
            && !matches!(join_rule, JoinRule::KnockRestricted(_)))
    {
        return Err(
            "join rule is not set to knock or knock_restricted, knocking is not allowed".to_owned()
        );
    }

    // Since v7, if sender does not match state_key, reject.
    if room_member_event.sender() != target_user {
        return Err("cannot make another user knock, sender does not match target user".to_owned());
    }

    let sender_membership = fetch_state.user_membership(room_member_event.sender())?;

    // Since v7, if the sender’s current membership is not ban, invite, or join, allow.
    // Otherwise, reject.
    if !matches!(
        sender_membership,
        MembershipState::Ban | MembershipState::Invite | MembershipState::Join
    ) {
        Ok(())
    } else {
        Err("cannot knock if user is banned, invited or joined".to_owned())
    }
}
