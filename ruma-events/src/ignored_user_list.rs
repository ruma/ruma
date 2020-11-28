//! Types for the *m.ignored_user_list* event.

use ruma_events_macros::BasicEventContent;
use ruma_identifiers::UserId;
use serde::{Deserialize, Serialize};

use crate::BasicEvent;

/// A list of users to ignore.
pub type IgnoredUserListEvent = BasicEvent<IgnoredUserListEventContent>;

/// The payload for `IgnoredUserListEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, BasicEventContent)]
#[ruma_event(type = "m.ignored_user_list")]
pub struct IgnoredUserListEventContent {
    /// A list of users to ignore.
    #[serde(with = "ruma_serde::vec_as_map_of_empty")]
    pub ignored_users: Vec<UserId>,
}

#[cfg(test)]
mod tests {
    use matches::assert_matches;
    use ruma_identifiers::user_id;
    use ruma_serde::Raw;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::IgnoredUserListEventContent;
    use crate::{AnyBasicEventContent, BasicEvent};

    #[test]
    fn serialization() {
        let ignored_user_list_event = BasicEvent {
            content: IgnoredUserListEventContent {
                ignored_users: vec![user_id!("@carl:example.com")],
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
            from_json_value::<Raw<BasicEvent<AnyBasicEventContent>>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            BasicEvent {
                content: AnyBasicEventContent::IgnoredUserList(IgnoredUserListEventContent {
                    ignored_users,
                }),
            } if ignored_users == vec![user_id!("@carl:example.com")]
        );
    }
}
