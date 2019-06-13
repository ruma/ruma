//! Types for the *m.ignored_user_list* event.

use std::collections::HashMap;

use ruma_identifiers::UserId;
use serde::{Deserialize, Serialize};

event! {
    /// A list of users to ignore.
    pub struct IgnoredUserListEvent(IgnoredUserListEventContent) {}
}

/// The payload of an `IgnoredUserListEvent`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct IgnoredUserListEventContent {
    /// A list of users to ignore.
    ///
    /// The values in the hash map are not meaningful. They are used to generate an empty JSON
    /// object to support the odd structure used by the Matrix specification:
    ///
    /// ```text
    /// "@someone:example.org": {}
    /// ```
    pub ignored_users: HashMap<UserId, HashMap<(), ()>>,
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, convert::TryFrom};

    use ruma_identifiers::UserId;

    use super::IgnoredUserListEventContent;

    #[test]
    fn serialize_to_empty_json_object() {
        let mut ignored_user_list_event_content = IgnoredUserListEventContent {
            ignored_users: HashMap::new(),
        };

        let user_id = UserId::try_from("@carl:example.com").unwrap();

        ignored_user_list_event_content
            .ignored_users
            .insert(user_id, HashMap::new());

        let json = serde_json::to_string(&ignored_user_list_event_content).unwrap();

        assert_eq!(json, r#"{"ignored_users":{"@carl:example.com":{}}}"#);
    }

    #[test]
    fn deserialize_from_empty_json_object() {
        let json = r#"{"ignored_users":{"@carl:example.com":{}}}"#;

        let ignored_user_list_event_content: IgnoredUserListEventContent =
            serde_json::from_str(&json).unwrap();

        let mut expected = IgnoredUserListEventContent {
            ignored_users: HashMap::new(),
        };

        let user_id = UserId::try_from("@carl:example.com").unwrap();

        expected.ignored_users.insert(user_id, HashMap::new());

        assert_eq!(ignored_user_list_event_content, expected);
    }
}
