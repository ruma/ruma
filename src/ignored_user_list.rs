//! Types for the *m.ignored_user_list* event.

use std::collections::HashMap;

use ruma_identifiers::UserId;
use serde::{ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};

use crate::{Empty, Event, EventResult, EventType, InnerInvalidEvent, InvalidEvent};

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

impl<'de> Deserialize<'de> for EventResult<IgnoredUserListEvent> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = serde_json::Value::deserialize(deserializer)?;

        let raw: raw::IgnoredUserListEvent = match serde_json::from_value(json.clone()) {
            Ok(raw) => raw,
            Err(error) => {
                return Ok(EventResult::Err(InvalidEvent(
                    InnerInvalidEvent::Validation {
                        json,
                        message: error.to_string(),
                    },
                )));
            }
        };

        Ok(EventResult::Ok(IgnoredUserListEvent {
            content: IgnoredUserListEventContent {
                ignored_users: raw.content.ignored_users.keys().cloned().collect(),
            },
        }))
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

impl_event!(
    IgnoredUserListEvent,
    IgnoredUserListEventContent,
    EventType::IgnoredUserList
);

impl<'de> Deserialize<'de> for EventResult<IgnoredUserListEventContent> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = serde_json::Value::deserialize(deserializer)?;

        let raw: raw::IgnoredUserListEventContent = match serde_json::from_value(json.clone()) {
            Ok(raw) => raw,
            Err(error) => {
                return Ok(EventResult::Err(InvalidEvent(
                    InnerInvalidEvent::Validation {
                        json,
                        message: error.to_string(),
                    },
                )));
            }
        };

        Ok(EventResult::Ok(IgnoredUserListEventContent {
            ignored_users: raw.ignored_users.keys().cloned().collect(),
        }))
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
    use std::convert::TryFrom;

    use ruma_identifiers::UserId;

    use super::{EventResult, IgnoredUserListEvent, IgnoredUserListEventContent};

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

        let actual = serde_json::from_str::<EventResult<IgnoredUserListEvent>>(json)
            .unwrap()
            .into_result()
            .unwrap();

        let expected = IgnoredUserListEvent {
            content: IgnoredUserListEventContent {
                ignored_users: vec![UserId::try_from("@carl:example.com").unwrap()],
            },
        };

        assert_eq!(actual, expected);
    }
}
