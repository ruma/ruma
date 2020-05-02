//! Types for the *m.ignored_user_list* event.

use ruma_identifiers::UserId;
use serde::{Deserialize, Serialize};

use crate::{EventType, FromRaw};

/// A list of users to ignore.
#[derive(Clone, Debug, Serialize)]
#[serde(rename = "m.ignored_user_list", tag = "type")]
pub struct IgnoredUserListEvent {
    /// The event's content.
    pub content: IgnoredUserListEventContent,
}

impl FromRaw for IgnoredUserListEvent {
    type Raw = raw::IgnoredUserListEvent;

    fn from_raw(raw: raw::IgnoredUserListEvent) -> Self {
        Self {
            content: FromRaw::from_raw(raw.content),
        }
    }
}

/// The payload for `IgnoredUserListEvent`.
#[derive(Clone, Debug, Serialize)]
pub struct IgnoredUserListEventContent {
    /// A list of users to ignore.
    #[serde(with = "ruma_serde::vec_as_map_of_empty")]
    pub ignored_users: Vec<UserId>,
}

impl FromRaw for IgnoredUserListEventContent {
    type Raw = raw::IgnoredUserListEventContent;

    fn from_raw(raw: raw::IgnoredUserListEventContent) -> Self {
        Self {
            ignored_users: raw.ignored_users,
        }
    }
}

impl_event!(
    IgnoredUserListEvent,
    IgnoredUserListEventContent,
    EventType::IgnoredUserList
);

pub(crate) mod raw {
    use super::*;

    /// A list of users to ignore.
    #[derive(Clone, Debug, Deserialize)]
    pub struct IgnoredUserListEvent {
        /// The event's content.
        pub content: IgnoredUserListEventContent,
    }

    /// The payload for `IgnoredUserListEvent`.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct IgnoredUserListEventContent {
        /// A list of users to ignore.
        #[serde(with = "ruma_serde::vec_as_map_of_empty")]
        pub ignored_users: Vec<UserId>,
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
