//! Types for the *m.room.name* event.

use js_int::UInt;
use serde::{Deserialize, Serialize};

use crate::empty_string_as_none;

state_event! {
    /// A human-friendly room name designed to be displayed to the end-user.
    pub struct NameEvent(NameEventContent) {}
}

/// The payload of a `NameEvent`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NameEventContent {
    /// The name of the room. This MUST NOT exceed 255 bytes.
    // The spec says “A room with an m.room.name event with an absent, null, or empty name field
    // should be treated the same as a room with no m.room.name event.”.
    // Serde maps null fields to None by default, serde(default) maps an absent field to None,
    // and empty_string_as_none completes the handling.
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::NameEventContent;
    use serde_json::from_str;

    #[test]
    fn absent_field_as_none() {
        assert_eq!(from_str::<NameEventContent>(r#"{}"#).unwrap().name, None);
    }

    #[test]
    fn null_field_as_none() {
        assert_eq!(
            from_str::<NameEventContent>(r#"{"name":null}"#)
                .unwrap()
                .name,
            None
        );
    }

    #[test]
    fn empty_field_as_none() {
        assert_eq!(
            from_str::<NameEventContent>(r#"{"name":""}"#).unwrap().name,
            None
        );
    }

    #[test]
    fn nonempty_field_as_some() {
        let name = Some("The room name".to_string());

        assert_eq!(
            from_str::<NameEventContent>(r##"{"name":"The room name"}"##)
                .unwrap()
                .name,
            name
        );
    }
}
