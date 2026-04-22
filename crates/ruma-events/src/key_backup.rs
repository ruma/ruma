//! Types for the [`m.key_backup`] account data event.
//!
//! [`m.key_backup`]: https://github.com/matrix-org/matrix-spec-proposals/pull/4287

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of an [`m.key_backup`] event.
///
/// [`m.key_backup`]: https://github.com/matrix-org/matrix-spec-proposals/pull/4287
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.key_backup", kind = GlobalAccountData)]
pub struct KeyBackupEventContent {
    /// Is key backup (key storage) explicitly enabled or disabled by the user?
    pub enabled: bool,
}

impl KeyBackupEventContent {
    /// Creates a new `KeyBackupEventContent`.
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::canonical_json::assert_to_canonical_json_eq;
    use serde_json::{from_value as from_json_value, json};

    use super::KeyBackupEventContent;
    use crate::AnyGlobalAccountDataEvent;

    #[test]
    fn key_backup_serialization() {
        let content_false = KeyBackupEventContent::new(false);

        assert_to_canonical_json_eq!(
            content_false,
            json!({
                "enabled": false,
            }),
        );

        let content_true = KeyBackupEventContent::new(true);

        assert_to_canonical_json_eq!(
            content_true,
            json!({
                "enabled": true,
            }),
        );
    }

    #[test]
    fn key_backup_deserialization() {
        let json_false = json!({
            "content": {
                "enabled": false,
            },
            "type": "m.key_backup",
        });

        assert_matches!(
            from_json_value::<AnyGlobalAccountDataEvent>(json_false),
            Ok(AnyGlobalAccountDataEvent::KeyBackup(ev_false))
        );

        assert!(!ev_false.content.enabled);

        let json_true = json!({
            "content": {
                "enabled": true,
            },
            "type": "m.key_backup",
        });

        assert_matches!(
            from_json_value::<AnyGlobalAccountDataEvent>(json_true),
            Ok(AnyGlobalAccountDataEvent::KeyBackup(ev_true))
        );

        assert!(ev_true.content.enabled);
    }
}
