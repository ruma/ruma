use super::MembershipState;
use crate::{MxcUri, UserId};

/// The details of a member event required to calculate a [`MembershipChange`].
#[derive(Clone, Debug)]
pub struct MembershipDetails<'a> {
    pub(crate) avatar_url: Option<&'a MxcUri>,
    pub(crate) displayname: Option<&'a str>,
    pub(crate) membership: &'a MembershipState,
}

/// Translation of the membership change in `m.room.member` event.
#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum MembershipChange<'a> {
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
        /// The details of the displayname change, if applicable.
        displayname_change: Option<Change<Option<&'a str>>>,

        /// The details of the avatar url change, if applicable.
        avatar_url_change: Option<Change<Option<&'a MxcUri>>>,
    },

    /// Not implemented.
    NotImplemented,
}

/// A simple representation of a change, containing old and new data.
#[derive(Clone, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct Change<T> {
    /// The old data.
    pub old: T,

    /// The new data.
    pub new: T,
}

impl<T: PartialEq> Change<T> {
    fn new(old: T, new: T) -> Option<Self> {
        if old == new {
            None
        } else {
            Some(Self { old, new })
        }
    }
}

/// Internal function so all `RoomMemberEventContent` state event kinds can share the same
/// implementation.
pub(super) fn membership_change<'a>(
    details: MembershipDetails<'a>,
    prev_details: Option<MembershipDetails<'a>>,
    sender: &UserId,
    state_key: &UserId,
) -> MembershipChange<'a> {
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
            displayname_change: Change::new(prev_details.displayname, details.displayname),
            avatar_url_change: Change::new(prev_details.avatar_url, details.avatar_url),
        },
        (St::Join, St::Leave) if sender == state_key => Ch::Left,
        (St::Join, St::Leave) => Ch::Kicked,
        (St::Join, St::Ban) => Ch::KickedAndBanned,
        (St::Leave, St::Invite) => Ch::Invited,
        (St::Ban, St::Leave) => Ch::Unbanned,
        _ => Ch::NotImplemented,
    }
}
