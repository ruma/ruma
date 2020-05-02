//! Types for the *m.ignored_user_list* event.

use ruma_events_macros::ruma_event;
use ruma_identifiers::UserId;

ruma_event! {
    /// A list of users to ignore.
    IgnoredUserListEvent {
        kind: Event,
        event_type: "m.ignored_user_list",
        content: {
            /// A list of users to ignore.
            #[serde(with = "ruma_serde::vec_as_map_of_empty")]
            pub ignored_users: Vec<UserId>,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use matches::assert_matches;
    use ruma_identifiers::UserId;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{IgnoredUserListEvent, IgnoredUserListEventContent};
    use crate::EventJson;

    #[test]
    fn serialization() {
        let ignored_user_list_event = IgnoredUserListEvent {
            content: IgnoredUserListEventContent {
                ignored_users: vec![UserId::try_from("@carl:example.com").unwrap()],
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
            from_json_value::<EventJson<IgnoredUserListEvent>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            IgnoredUserListEvent {
                content: IgnoredUserListEventContent { ignored_users, },
            } if ignored_users == vec![UserId::try_from("@carl:example.com").unwrap()]
        );
    }
}
