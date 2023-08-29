//! Common types for [spaces].
//!
//! [spaces]: https://spec.matrix.org/latest/client-server-api/#spaces

use ruma_macros::StringEnum;

use crate::PrivOwnedStr;

/// The rule used for users wishing to join a room.
///
/// In contrast to the regular `JoinRule` in `ruma_events`, this enum does not hold the conditions
/// for joining restricted rooms. Instead, the server is assumed to only return rooms the user is
/// allowed to join in a space hierarchy listing response.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Default, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum SpaceRoomJoinRule {
    /// A user who wishes to join the room must first receive an invite to the room from someone
    /// already inside of the room.
    Invite,

    /// Users can join the room if they are invited, or they can request an invite to the room.
    ///
    /// They can be allowed (invited) or denied (kicked/banned) access.
    Knock,

    /// Reserved but not yet implemented by the Matrix specification.
    Private,

    /// Users can join the room if they are invited, or if they meet any of the conditions
    /// described in a set of allow rules.
    ///
    /// These rules are not made available as part of a space hierarchy listing response and can
    /// only be seen by users inside the room.
    Restricted,

    /// Users can join the room if they are invited, or if they meet any of the conditions
    /// described in a set of allow rules, or they can request an invite to the room.
    KnockRestricted,

    /// Anyone can join the room without any prior action.
    #[default]
    Public,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
