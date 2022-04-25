use serde::{Deserialize, Serialize};

use super::MembershipState;
use crate::{MxcUri, UserId};

/// The details of a member event required to calculate a [`MembershipChange`].
pub struct MembershipDetails<'a> {
    pub(crate) avatar_url: Option<&'a MxcUri>,
    pub(crate) displayname: Option<&'a str>,
    pub(crate) membership: &'a MembershipState,
}

/// Translation of the membership change in `m.room.member` event.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum MembershipChange {
    /// No change.
    None,

    /// Must never happen.
    Error,

    /// User joined the room.
    Joined,

    /// User left the room.
    Left,

    /// User was banned.
    Banned,

    /// User was unbanned.
    Unbanned,

    /// User was kicked.
    Kicked,

    /// User was invited.
    Invited,

    /// User was kicked and banned.
    KickedAndBanned,

    /// User rejected the invite.
    InvitationRejected,

    /// User had their invite revoked.
    InvitationRevoked,

    /// `displayname` or `avatar_url` changed.
    ProfileChanged {
        /// Whether the `displayname` changed.
        displayname_changed: bool,

        /// Whether the `avatar_url` changed.
        avatar_url_changed: bool,
    },

    /// Not implemented.
    NotImplemented,
}

/// Internal function so all `RoomMemberEventContent` state event kinds can share the same
/// implementation.
pub(super) fn membership_change(
    details: MembershipDetails<'_>,
    prev_details: Option<MembershipDetails<'_>>,
    sender: &UserId,
    state_key: &UserId,
) -> MembershipChange {
    use MembershipChange as Ch;
    use MembershipState as St;

    let prev_details = match prev_details {
        Some(prev) => prev,
        None => MembershipDetails { avatar_url: None, displayname: None, membership: &St::Leave },
    };

    match (&prev_details.membership, &details.membership) {
        (St::Invite, St::Invite) | (St::Leave, St::Leave) | (St::Ban, St::Ban) => Ch::None,
        (St::Invite, St::Join) | (St::Leave, St::Join) => Ch::Joined,
        (St::Invite, St::Leave) if sender == state_key => Ch::InvitationRevoked,
        (St::Invite, St::Leave) => Ch::InvitationRejected,
        (St::Invite, St::Ban) | (St::Leave, St::Ban) => Ch::Banned,
        (St::Join, St::Invite) | (St::Ban, St::Invite) | (St::Ban, St::Join) => Ch::Error,
        (St::Join, St::Join) => Ch::ProfileChanged {
            displayname_changed: prev_details.displayname != details.displayname,
            avatar_url_changed: prev_details.avatar_url != details.avatar_url,
        },
        (St::Join, St::Leave) if sender == state_key => Ch::Left,
        (St::Join, St::Leave) => Ch::Kicked,
        (St::Join, St::Ban) => Ch::KickedAndBanned,
        (St::Leave, St::Invite) => Ch::Invited,
        (St::Ban, St::Leave) => Ch::Unbanned,
        _ => Ch::NotImplemented,
    }
}
