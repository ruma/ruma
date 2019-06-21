//! Types for the *m.ignored_user_list* event.

use std::collections::HashMap;

use ruma_identifiers::UserId;
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};

use crate::{Empty, Event, EventType};

/// A list of users to ignore.
#[derive(Clone, Debug, PartialEq)]
pub struct IgnoredUserListEvent {
    /// The event's content.
    pub content: IgnoredUserListEventContent,
}

/// The payload for `IgnoredUserListEvent`.
#[derive(Clone, Debug, PartialEq)]
pub struct IgnoredUserListEventContent {
    /// A list of users to ignore.
    pub ignored_users: Vec<UserId>,
}

impl IgnoredUserListEvent {
    /// Attempt to create `Self` from parsing a string of JSON data.
    pub fn from_str(json: &str) -> Result<Self, crate::InvalidEvent> {
        let raw = serde_json::from_str::<raw::IgnoredUserListEvent>(json)?;

        Ok(Self {
            content: IgnoredUserListEventContent {
                ignored_users: raw.content.ignored_users.keys().cloned().collect(),
            },
        })
    }
}

impl Serialize for IgnoredUserListEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("IgnoredUserListEvent", 2)?;

        state.serialize_field("content", &self.content)?;
        state.serialize_field("type", &self.event_type())?;

        state.end()
    }
}

impl crate::Event for IgnoredUserListEvent {
    /// The type of this event's `content` field.
    type Content = IgnoredUserListEventContent;

    /// The event's content.
    fn content(&self) -> &Self::Content {
        &self.content
    }

    /// The type of the event.
    fn event_type(&self) -> EventType {
        EventType::IgnoredUserList
    }
}

impl Serialize for IgnoredUserListEventContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = HashMap::new();

        for user_id in &self.ignored_users {
            map.insert(user_id.clone(), Empty);
        }

        let raw = raw::IgnoredUserListEventContent { ignored_users: map };

        raw.serialize(serializer)
    }
}

mod raw {
    use super::*;
    use crate::Empty;

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
        pub ignored_users: HashMap<UserId, Empty>,
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, convert::TryFrom};

    use ruma_identifiers::UserId;

    use super::{IgnoredUserListEvent, IgnoredUserListEventContent};

    #[test]
    fn serialization() {
        let ignored_user_list_event = IgnoredUserListEvent {
            content: IgnoredUserListEventContent {
                ignored_users: vec![UserId::try_from("@carl:example.com").unwrap()],
            },
        };

        let json = serde_json::to_string(&ignored_user_list_event).unwrap();

        assert_eq!(json, r#"{"content":{"ignored_users":{"@carl:example.com":{}}},"type":"m.ignored_user_list"}"#);
    }

    #[test]
    fn deserialization() {
        let json = r#"{"content":{"ignored_users":{"@carl:example.com":{}}},"type":"m.ignored_user_list"}"#;

        let actual = IgnoredUserListEvent::from_str(json).unwrap();

        let expected = IgnoredUserListEvent {
            content: IgnoredUserListEventContent {
                ignored_users: vec![UserId::try_from("@carl:example.com").unwrap()],
            },
        };

        assert_eq!(actual, expected);
    }
}
