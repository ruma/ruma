//! Types for the [`m.invite_permission_config`] account data.
//!
//! [`m.invite_permission_config`]: https://github.com/matrix-org/matrix-spec-proposals/pull/4380

use ruma_macros::{EventContent, StringEnum};
use serde::{Deserialize, Serialize};

use crate::PrivOwnedStr;

/// The content of an [`m.invite_permission_config`] account data.
///
/// Controls whether invites to this account are permitted.
///
/// [`m.invite_permission_config`]: https://github.com/matrix-org/matrix-spec-proposals/pull/4380
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(
    kind = GlobalAccountData,
    type = "m.invite_permission_config",
)]
pub struct InvitePermissionConfigEventContent {
    /// The default action chosen by the user that the homeserver should perform automatically when
    /// receiving an invitation for this account.
    ///
    /// A missing, invalid or unsupported value means that the user wants to receive invites as
    /// normal. Other parts of the specification might still have effects on invites, like
    /// [ignoring users].
    ///
    /// [ignoring users]: https://spec.matrix.org/latest/client-server-api/#ignoring-users
    #[serde(
        default,
        deserialize_with = "ruma_common::serde::default_on_error",
        skip_serializing_if = "Option::is_none"
    )]
    pub default_action: Option<InvitePermissionAction>,
}

impl InvitePermissionConfigEventContent {
    /// Creates a new empty `InvitePermissionConfigEventContent`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Possible actions in response to an invite.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[ruma_enum(rename_all = "lowercase")]
#[non_exhaustive]
pub enum InvitePermissionAction {
    /// Reject the invite.
    Block,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The content of an [`org.matrix.msc4380.invite_permission_config`][MSC4380] account data, the
/// unstable version of [`InvitePermissionConfigEventContent`].
///
/// Controls whether invites to this account are permitted.
///
/// [MSC4380]: https://github.com/matrix-org/matrix-spec-proposals/pull/4380
#[cfg(feature = "unstable-msc4380")]
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(
    kind = GlobalAccountData,
    type = "org.matrix.msc4380.invite_permission_config",
)]
pub struct UnstableInvitePermissionConfigEventContent {
    /// When set to true, indicates that the user does not wish to receive *any* room invites, and
    /// they should be blocked.
    #[serde(default, deserialize_with = "ruma_common::serde::default_on_error")]
    pub block_all: bool,
}

#[cfg(feature = "unstable-msc4380")]
impl UnstableInvitePermissionConfigEventContent {
    /// Creates a new `UnstableInvitePermissionConfigEventContent` from the desired boolean state.
    pub fn new(block_all: bool) -> Self {
        Self { block_all }
    }
}

#[cfg(feature = "unstable-msc4380")]
impl From<UnstableInvitePermissionConfigEventContent> for InvitePermissionConfigEventContent {
    fn from(value: UnstableInvitePermissionConfigEventContent) -> Self {
        Self { default_action: value.block_all.then_some(InvitePermissionAction::Block) }
    }
}

#[cfg(feature = "unstable-msc4380")]
impl From<InvitePermissionConfigEventContent> for UnstableInvitePermissionConfigEventContent {
    fn from(value: InvitePermissionConfigEventContent) -> Self {
        Self {
            block_all: value
                .default_action
                .is_some_and(|action| matches!(action, InvitePermissionAction::Block)),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::canonical_json::assert_to_canonical_json_eq;
    use serde_json::{from_value as from_json_value, json};

    #[cfg(feature = "unstable-msc4380")]
    use super::UnstableInvitePermissionConfigEventContent;
    use super::{InvitePermissionAction, InvitePermissionConfigEventContent};
    use crate::AnyGlobalAccountDataEvent;

    #[cfg(feature = "unstable-msc4380")]
    #[test]
    fn unstable_serialization() {
        let invite_permission_config = UnstableInvitePermissionConfigEventContent::new(true);

        assert_to_canonical_json_eq!(
            invite_permission_config,
            json!({
                "block_all": true,
            }),
        );
    }

    #[cfg(feature = "unstable-msc4380")]
    #[test]
    fn unstable_deserialization() {
        let json = json!({
            "content": {
                "block_all": true,
            },
            "type": "org.matrix.msc4380.invite_permission_config",
        });

        assert_matches!(
            from_json_value::<AnyGlobalAccountDataEvent>(json),
            Ok(AnyGlobalAccountDataEvent::UnstableInvitePermissionConfig(ev))
        );
        assert!(ev.content.block_all);
    }

    #[test]
    fn stable_serialization() {
        let mut invite_permission_config = InvitePermissionConfigEventContent::new();
        assert_to_canonical_json_eq!(invite_permission_config, json!({}),);

        invite_permission_config.default_action = Some(InvitePermissionAction::Block);
        assert_to_canonical_json_eq!(
            invite_permission_config,
            json!({
                "default_action": "block",
            }),
        );
    }

    #[test]
    fn stable_deserialization() {
        let json = json!({
            "content": {
                "default_action": "block",
            },
            "type": "m.invite_permission_config",
        });
        assert_matches!(
            from_json_value::<AnyGlobalAccountDataEvent>(json),
            Ok(AnyGlobalAccountDataEvent::InvitePermissionConfig(ev))
        );
        assert_eq!(ev.content.default_action, Some(InvitePermissionAction::Block));

        let json = json!({
            "content": {},
            "type": "m.invite_permission_config",
        });
        assert_matches!(
            from_json_value::<AnyGlobalAccountDataEvent>(json),
            Ok(AnyGlobalAccountDataEvent::InvitePermissionConfig(ev))
        );
        assert_eq!(ev.content.default_action, None);
    }
}
