//! Types for the [`m.ignored_user_list`] event.
//!
//! [`m.ignored_user_list`]: https://spec.matrix.org/v1.2/client-server-api/#mignored_user_list

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::OwnedUserId;

/// The content of an `m.ignored_user_list` event.
///
/// A list of users to ignore.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.ignored_user_list", kind = GlobalAccountData)]
pub struct IgnoredUserListEventContent {
    /// A list of users to ignore.
    #[serde(with = "crate::serde::vec_as_map_of_empty")]
    pub ignored_users: Vec<OwnedUserId>,
}

impl IgnoredUserListEventContent {
    /// Creates a new `IgnoredUserListEventContent` from the given user IDs.
    pub fn new(ignored_users: Vec<OwnedUserId>) -> Self {
        Self { ignored_users }
    }
}

#[cfg(test)]
mod tests {
    use crate::user_id;
    use assert_matches::assert_matches;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{IgnoredUserListEvent, IgnoredUserListEventContent};
    use crate::events::{AnyGlobalAccountDataEvent, GlobalAccountDataEvent};

    #[test]
    fn serialization() {
        let ignored_user_list_event = GlobalAccountDataEvent {
            content: IgnoredUserListEventContent {
                ignored_users: vec![user_id!("@carl:example.com").to_owned()],
            },
        };

        let json = json!({
            "content": {
                "ignored_users": {
                    "@carl:example.com": {}
                }
            },
            "type": "m.ignored_user_list"
        });

        assert_eq!(to_json_value(ignored_user_list_event).unwrap(), json);
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
            from_json_value::<AnyGlobalAccountDataEvent>(json).unwrap(),
            AnyGlobalAccountDataEvent::IgnoredUserList(
                IgnoredUserListEvent {
                    content: IgnoredUserListEventContent {
                        ignored_users
                    },
                })
         if ignored_users == vec![user_id!("@carl:example.com")]
        );
    }
}
