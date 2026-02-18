//! Types for the [`m.room.language`] event.
//!
//! [`m.room.language`]: https://github.com/matrix-org/matrix-spec-proposals/pull/4334

use language_tags::LanguageTag;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::EmptyStateKey;

/// The content of an `org.matrix.msc4334.room.language` event.
///
/// The room language is a [IETF BCP 47](https://datatracker.ietf.org/doc/bcp47/) language code.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc4334.room.language", kind = State, state_key_type = EmptyStateKey)]
pub struct RoomLanguageEventContent {
    /// The language of the room.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<LanguageTag>,
}

impl RoomLanguageEventContent {
    /// Create a new `RoomLanguageEventContent` with the given language.
    pub fn new(language: LanguageTag) -> Self {
        Self { language: Some(language) }
    }
}

#[cfg(test)]
mod tests {
    use ruma_common::canonical_json::assert_to_canonical_json_eq;
    use serde_json::{from_value as from_json_value, json};

    use super::RoomLanguageEventContent;
    use crate::{StateEvent, room::language::LanguageTag};

    #[test]
    fn serialization() {
        let content = RoomLanguageEventContent { language: LanguageTag::parse("fr").ok() };

        assert_to_canonical_json_eq!(
            content,
            json!({
                "language": "fr",
            }),
        );
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
            from_json_value::<StateEvent<RoomLanguageEventContent>>(json_data)
                .unwrap()
                .content
                .language,
            Some(LanguageTag::parse("fr").unwrap())
        );
    }
}
