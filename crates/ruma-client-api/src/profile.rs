//! Endpoints for user profiles.

#[cfg(feature = "client")]
use ruma_common::{
    api::{
        MatrixVersion,
        path_builder::{StablePathSelector, VersionHistory},
    },
    profile::ProfileFieldName,
};

#[cfg(feature = "unstable-msc4466")]
use crate::PrivOwnedStr;
#[cfg(feature = "unstable-msc4466")]
use ruma_common::serde::StringEnum;

pub mod delete_profile_field;
pub mod get_avatar_url;
pub mod get_display_name;
pub mod get_profile;
pub mod get_profile_field;
#[cfg(feature = "client")]
mod profile_field_serde;
pub mod set_avatar_url;
pub mod set_display_name;
pub mod set_profile_field;
mod static_profile_field;

pub use self::static_profile_field::*;

/// Endpoint version history valid only for profile fields that didn't exist before Matrix 1.16.
#[cfg(feature = "client")]
const EXTENDED_PROFILE_FIELD_HISTORY: VersionHistory = VersionHistory::new(
    &[(
        Some("uk.tcpip.msc4133"),
        "/_matrix/client/unstable/uk.tcpip.msc4133/profile/{user_id}/{field}",
    )],
    &[(
        StablePathSelector::Version(MatrixVersion::V1_16),
        "/_matrix/client/v3/profile/{user_id}/{field}",
    )],
    None,
    None,
);

/// Whether the given field name existed already before custom fields were officially supported in
/// profiles.
#[cfg(feature = "client")]
fn field_existed_before_extended_profiles(field_name: &ProfileFieldName) -> bool {
    matches!(field_name, ProfileFieldName::AvatarUrl | ProfileFieldName::DisplayName)
}

/// Controls which rooms the server should send an updated `m.room.member` event in
/// when changing `displayname` or `avatar_url` in a user's profile. Defined by [MSC4466][1].
///
/// [1]: https://github.com/matrix-org/matrix-spec-proposals/pull/4466
#[cfg(feature = "unstable-msc4466")]
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Default, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum PropagateTo {
    /// The server must send a `m.room.member` event in all of the user's
    /// joined rooms.
    #[default]
    All,

    /// The server must only send a `m.room.member` event in rooms where the profile
    /// field being updated does _not_ differ from its value in the user's global profile data.
    Unchanged,

    /// The server must not send a `m.room.member` event to any rooms.
    None,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
