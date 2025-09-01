//! Types for the [`m.room.language`] event.
//!
//! [`m.room.language`]: https://github.com/matrix-org/matrix-spec-proposals/pull/4334

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::EmptyStateKey;

/// The content of an `m.room.language` event.
///
/// The room language is a [IETF BCP 47](https://developer.mozilla.org/en-US/docs/Glossary/BCP_47_language_tag) language code.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc4334.room.language", kind = State, state_key_type = EmptyStateKey)]
pub struct RoomLanguageEventContent {
    /// The language of the room.
    pub language: String,
}

impl RoomLanguageEventContent {
    /// Create a new `RoomLanguageEventContent` with the given language.
    pub fn new(language: String) -> Self {
        Self { language }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::RoomLanguageEventContent;
    use crate::OriginalStateEvent;

    #[test]
    fn serialization() {
        let content = RoomLanguageEventContent { language: "fr".to_owned() };

        let actual = to_json_value(content).unwrap();
        let expected = json!({
            "language": "fr",
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn deserialization() {
        let json_data = json!({
            "content": {
                "language": "fr"
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "org.matrix.msc4334.room.language"
        });

        assert_eq!(
            from_json_value::<OriginalStateEvent<RoomLanguageEventContent>>(json_data)
                .unwrap()
                .content
                .language,
            "fr"
        );
    }
}
