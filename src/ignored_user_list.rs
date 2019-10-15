//! Types for the *m.ignored_user_list* event.

use ruma_identifiers::UserId;
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};

use crate::{vec_as_map_of_empty, Event as _, EventType, TryFromRaw, Void};

/// A list of users to ignore.
#[derive(Clone, Debug, PartialEq)]
pub struct IgnoredUserListEvent {
    /// The event's content.
    pub content: IgnoredUserListEventContent,
}

impl TryFromRaw for IgnoredUserListEvent {
    type Raw = raw::IgnoredUserListEvent;
    type Err = Void;

    fn try_from_raw(raw: raw::IgnoredUserListEvent) -> Result<Self, (Self::Err, Self::Raw)> {
        Ok(Self {
            content: crate::from_raw(raw.content),
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

/// The payload for `IgnoredUserListEvent`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct IgnoredUserListEventContent {
    /// A list of users to ignore.
    pub ignored_users: Vec<UserId>,
}

impl TryFromRaw for IgnoredUserListEventContent {
    type Raw = raw::IgnoredUserListEventContent;
    type Err = Void;

    fn try_from_raw(raw: raw::IgnoredUserListEventContent) -> Result<Self, (Self::Err, Self::Raw)> {
        Ok(Self {
            ignored_users: raw.ignored_users,
        })
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
        #[serde(with = "vec_as_map_of_empty")]
        pub ignored_users: Vec<UserId>,
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use ruma_identifiers::UserId;

    use super::{IgnoredUserListEvent, IgnoredUserListEventContent};
    use crate::EventResult;

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
