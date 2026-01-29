//! Types for the [`m.invite_permission_config`] account data event.
//!
//! [`m.invite_permission_config`]: https://github.com/matrix-org/matrix-spec-proposals/pull/4380

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of an `m.invite_permission_config` event.
///
/// A single property: `block_all`.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(
    kind = GlobalAccountData,
    type = "org.matrix.msc4380.invite_permission_config",
    alias = "m.invite_permission_config",
)]
pub struct InvitePermissionConfigEventContent {
    /// When set to true, indicates that the user does not wish to receive *any* room invites, and
    /// they should be blocked.
    #[serde(default)]
    #[serde(deserialize_with = "ruma_common::serde::default_on_error")]
    pub block_all: bool,
}

impl InvitePermissionConfigEventContent {
    /// Creates a new `InvitePermissionConfigEventContent` from the desired boolean state.
    pub fn new(block_all: bool) -> Self {
        Self { block_all }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::canonical_json::assert_to_canonical_json_eq;
    use serde_json::{from_value as from_json_value, json};

    use super::InvitePermissionConfigEventContent;
    use crate::AnyGlobalAccountDataEvent;

    #[test]
    fn serialization() {
        let invite_permission_config = InvitePermissionConfigEventContent::new(true);

        assert_to_canonical_json_eq!(
            invite_permission_config,
            json!({
                "block_all": true,
            }),
        );
    }

    #[test]
    fn deserialization() {
        let json = json!({
            "content": {
                "block_all": true
            },
            "type": "m.invite_permission_config"
        });

        assert_matches!(
            from_json_value::<AnyGlobalAccountDataEvent>(json),
            Ok(AnyGlobalAccountDataEvent::InvitePermissionConfig(ev))
        );
        assert!(ev.content.block_all);
    }
}
