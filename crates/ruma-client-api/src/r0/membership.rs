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

use ruma_common::thirdparty::Medium;
use ruma_identifiers::{ServerName, ServerSigningKeyId, UserId};
use ruma_serde::Outgoing;
use serde::Serialize;

/// A signature of an `m.third_party_invite` token to prove that this user owns a third party
/// identity which has been invited to the room.
#[derive(Clone, Debug, Outgoing, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ThirdPartySigned<'a> {
    /// The Matrix ID of the user who issued the invite.
    pub sender: &'a UserId,

    /// The Matrix ID of the invitee.
    pub mxid: &'a UserId,

    /// The state key of the `m.third_party_invite` event.
    pub token: &'a str,

    /// A signatures object containing a signature of the entire signed object.
    pub signatures: BTreeMap<Box<ServerName>, BTreeMap<ServerSigningKeyId, String>>,
}

impl<'a> ThirdPartySigned<'a> {
    /// Creates a new `ThirdPartySigned` from the given sender and invitee user IDs, state key token
    /// and signatures.
    pub fn new(
        sender: &'a UserId,
        mxid: &'a UserId,
        token: &'a str,
        signatures: BTreeMap<Box<ServerName>, BTreeMap<ServerSigningKeyId, String>>,
    ) -> Self {
        Self { sender, mxid, token, signatures }
    }
}

/// Represents third party IDs to invite to the room.
///
/// To create an instance of this type, first create a `Invite3pidInit` and convert it via
/// `Invite3pid::from` / `.into()`.
#[derive(Clone, Debug, PartialEq, Outgoing, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[incoming_derive(PartialEq)]
pub struct Invite3pid<'a> {
    /// Hostname and port of identity server to be used for account lookups.
    pub id_server: &'a str,

    /// An access token registered with the identity server.
    pub id_access_token: &'a str,

    /// Type of third party ID.
    pub medium: Medium,

    /// Third party identifier.
    pub address: &'a str,
}

/// Initial set of fields of `Invite3pid`.
///
/// This struct will not be updated even if additional fields are added to `Invite3pid` in a new
/// (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct Invite3pidInit<'a> {
    /// Hostname and port of identity server to be used for account lookups.
    pub id_server: &'a str,

    /// An access token registered with the identity server.
    pub id_access_token: &'a str,

    /// Type of third party ID.
    pub medium: Medium,

    /// Third party identifier.
    pub address: &'a str,
}

impl<'a> From<Invite3pidInit<'a>> for Invite3pid<'a> {
    fn from(init: Invite3pidInit<'a>) -> Self {
        let Invite3pidInit { id_server, id_access_token, medium, address } = init;
        Self { id_server, id_access_token, medium, address }
    }
}
