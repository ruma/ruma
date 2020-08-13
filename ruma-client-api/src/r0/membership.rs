//! Endpoints for room membership.

pub mod ban_user;
pub mod forget_room;
pub mod get_member_events;
pub mod invite_user;
pub mod join_room_by_id;
pub mod join_room_by_id_or_alias;
pub mod joined_members;
pub mod joined_rooms;
pub mod kick_user;
pub mod leave_room;
pub mod unban_user;

use std::collections::BTreeMap;

use ruma_api::Outgoing;
use ruma_common::thirdparty::Medium;
use ruma_identifiers::{ServerKeyId, ServerName};
use serde::{Deserialize, Serialize};

/// A signature of an `m.third_party_invite` token to prove that this user owns a third party
/// identity which has been invited to the room.
#[derive(Clone, Debug, Outgoing, Serialize)]
pub struct ThirdPartySigned<'a> {
    /// The Matrix ID of the user who issued the invite.
    pub sender: &'a str,

    /// The Matrix ID of the invitee.
    pub mxid: &'a str,

    /// The state key of the m.third_party_invite event.
    pub token: &'a str,

    /// A signatures object containing a signature of the entire signed object.
    pub signatures: BTreeMap<Box<ServerName>, BTreeMap<ServerKeyId, String>>,
}

/// Represents third party IDs to invite to the room.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Invite3pid {
    /// Hostname and port of identity server to be used for account lookups.
    pub id_server: String,

    /// An access token registered with the identity server.
    pub id_access_token: String,

    /// Type of third party ID.
    pub medium: Medium,

    /// Third party identifier.
    pub address: String,
}
