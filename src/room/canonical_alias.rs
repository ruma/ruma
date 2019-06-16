//! Types for the *m.room.canonical_alias* event.

use js_int::UInt;
use ruma_identifiers::RoomAliasId;
use serde::{Deserialize, Serialize};

use crate::empty_string_as_none;

state_event! {
    /// Informs the room as to which alias is the canonical one.
    pub struct CanonicalAliasEvent(CanonicalAliasEventContent) {}
}

/// The payload of a `CanonicalAliasEvent`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CanonicalAliasEventContent {
    /// The canonical alias.
    /// Rooms with `alias: None` should be treated the same as a room with no canonical alias.
    // The spec says “A room with an m.room.canonical_alias event with an absent, null, or empty alias field
    // should be treated the same as a room with no m.room.canonical_alias event.”.
    // Serde maps null fields to None by default, serde(default) maps an absent field to None,
    // and empty_string_as_none does exactly that, preventing empty strings getting parsed as RoomAliasId.
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub alias: Option<RoomAliasId>,
}

#[cfg(test)]
mod tests {
    use serde_json::from_str;

    use super::CanonicalAliasEventContent;
    use ruma_identifiers::RoomAliasId;
    use std::convert::TryFrom;

    #[test]
    fn absent_field_as_none() {
        assert_eq!(
            from_str::<CanonicalAliasEventContent>(r#"{}"#)
                .unwrap()
                .alias,
            None
        );
    }

    #[test]
    fn null_field_as_none() {
        assert_eq!(
            from_str::<CanonicalAliasEventContent>(r#"{"alias":null}"#)
                .unwrap()
                .alias,
            None
        );
    }

    #[test]
    fn empty_field_as_none() {
        assert_eq!(
            from_str::<CanonicalAliasEventContent>(r#"{"alias":""}"#)
                .unwrap()
                .alias,
            None
        );
    }

    #[test]
    fn nonempty_field_as_some() {
        let alias = Some(RoomAliasId::try_from("#somewhere:localhost").unwrap());

        assert_eq!(
            from_str::<CanonicalAliasEventContent>(r##"{"alias":"#somewhere:localhost"}"##)
                .unwrap()
                .alias,
            alias
        );
    }
}
