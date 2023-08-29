//! Types for the [`m.ignored_user_list`] event.
//!
//! [`m.ignored_user_list`]: https://spec.matrix.org/latest/client-server-api/#mignored_user_list

use std::collections::BTreeMap;

use ruma_common::OwnedUserId;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of an `m.ignored_user_list` event.
///
/// A list of users to ignore.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.ignored_user_list", kind = GlobalAccountData)]
pub struct IgnoredUserListEventContent {
    /// A map of users to ignore.
    ///
    /// As [`IgnoredUser`] is currently empty, only the user IDs are useful and
    /// can be accessed with the `.keys()` and `into_keys()` iterators.
    pub ignored_users: BTreeMap<OwnedUserId, IgnoredUser>,
}

impl IgnoredUserListEventContent {
    /// Creates a new `IgnoredUserListEventContent` from the given map of ignored user.
    pub fn new(ignored_users: BTreeMap<OwnedUserId, IgnoredUser>) -> Self {
        Self { ignored_users }
    }

    /// Creates a new `IgnoredUserListEventContent` from the given list of users.
    pub fn users(ignored_users: impl IntoIterator<Item = OwnedUserId>) -> Self {
        Self::new(ignored_users.into_iter().map(|id| (id, IgnoredUser {})).collect())
    }
}

/// Details about an ignored user.
///
/// This is currently empty.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct IgnoredUser {}

impl IgnoredUser {
    /// Creates an empty `IgnoredUser`.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::{owned_user_id, user_id};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::IgnoredUserListEventContent;
    use crate::AnyGlobalAccountDataEvent;

    #[test]
    fn serialization() {
        let ignored_user_list =
            IgnoredUserListEventContent::users(vec![owned_user_id!("@carl:example.com")]);

        let json = json!({
            "ignored_users": {
                "@carl:example.com": {}
            },
        });

        assert_eq!(to_json_value(ignored_user_list).unwrap(), json);
    }

    #[test]
    fn deserialization() {
        let json = json!({
            "content": {
                "ignored_users": {
                    "@carl:example.com": {}
                }
            },
            "type": "m.ignored_user_list"
        });

        assert_matches!(
            from_json_value::<AnyGlobalAccountDataEvent>(json),
            Ok(AnyGlobalAccountDataEvent::IgnoredUserList(ev))
        );
        assert_eq!(
            ev.content.ignored_users.keys().collect::<Vec<_>>(),
            vec![user_id!("@carl:example.com")]
        );
    }
}
