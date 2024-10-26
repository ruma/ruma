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
#[cfg(feature = "unstable-msc2666")]
pub mod mutual_rooms;
pub mod unban_user;

use ruma_common::{thirdparty::Medium, OwnedUserId, ServerSignatures};
use serde::{Deserialize, Serialize};

/// A signature of an `m.third_party_invite` token to prove that this user owns a third party
/// identity which has been invited to the room.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ThirdPartySigned {
    /// The Matrix ID of the user who issued the invite.
    pub sender: OwnedUserId,

    /// The Matrix ID of the invitee.
    pub mxid: OwnedUserId,

    /// The state key of the `m.third_party_invite` event.
    pub token: String,

    /// A signatures object containing a signature of the entire signed object.
    pub signatures: ServerSignatures,
}

impl ThirdPartySigned {
    /// Creates a new `ThirdPartySigned` from the given sender and invitee user IDs, state key token
    /// and signatures.
    pub fn new(
        sender: OwnedUserId,
        mxid: OwnedUserId,
        token: String,
        signatures: ServerSignatures,
    ) -> Self {
        Self { sender, mxid, token, signatures }
    }
}

/// Represents third party IDs to invite to the room.
///
/// To create an instance of this type, first create a `Invite3pidInit` and convert it via
/// `Invite3pid::from` / `.into()`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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

/// Initial set of fields of `Invite3pid`.
///
/// This struct will not be updated even if additional fields are added to `Invite3pid` in a new
/// (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct Invite3pidInit {
    /// Hostname and port of identity server to be used for account lookups.
    pub id_server: String,

    /// An access token registered with the identity server.
    pub id_access_token: String,

    /// Type of third party ID.
    pub medium: Medium,

    /// Third party identifier.
    pub address: String,
}

impl From<Invite3pidInit> for Invite3pid {
    fn from(init: Invite3pidInit) -> Self {
        let Invite3pidInit { id_server, id_access_token, medium, address } = init;
        Self { id_server, id_access_token, medium, address }
    }
}
