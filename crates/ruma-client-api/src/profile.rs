//! Endpoints for user profiles.

#[cfg(feature = "client")]
use ruma_common::{
    api::{
        MatrixVersion,
        path_builder::{StablePathSelector, VersionHistory},
    },
    profile::ProfileFieldName,
};

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
