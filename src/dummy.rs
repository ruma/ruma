//! Types for the *m.dummy* event.

use std::{
    collections::HashMap,
    fmt::{Formatter, Result as FmtResult},
};

use serde::{de::{Error, MapAccess, Visitor}, ser::{SerializeMap, SerializeStruct}, Deserialize, Deserializer, Serialize, Serializer};

use crate::Event;

/// This event type is used to indicate new Olm sessions for end-to-end encryption.
///
/// Typically it is encrypted as an *m.room.encrypted* event, then sent as a to-device event.
///
/// The event does not have any content associated with it. The sending client is expected to
/// send a key share request shortly after this message, causing the receiving client to process
/// this *m.dummy* event as the most recent event and using the keyshare request to set up the
/// session. The keyshare request and *m.dummy* combination should result in the original
/// sending client receiving keys over the newly established session.
#[derive(Clone, Debug)]
pub struct DummyEvent {
    /// The event's content.
    pub content: DummyEventContent,
}

/// The payload for `DummyEvent`.
#[derive(Clone, Debug)]
pub struct DummyEventContent;

impl DummyEvent {
    /// Attempt to create `Self` from parsing a string of JSON data.
    pub fn from_str(json: &str) -> Result<Self, crate::InvalidEvent> {
        serde_json::from_str::<raw::DummyEvent>(json)?;

        Ok(Self {
            content: DummyEventContent,
        })
    }
}

impl Serialize for DummyEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let mut state = serializer.serialize_struct("DummyEvent", 2)?;

        state.serialize_field("content", &self.content);
        state.serialize_field("type", &self.event_type());

        state.end()
    }
}

impl crate::Event for DummyEvent {
    /// The type of the event.
    const EVENT_TYPE: crate::EventType = crate::EventType::Dummy;

    /// The type of this event's `content` field.
    type Content = DummyEventContent;

    /// The event's content.
    fn content(&self) -> &Self::Content {
        &self.content
    }
}

// This is necessary because the content is represented in JSON as an empty object.
impl Serialize for DummyEventContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        serializer.serialize_map(Some(0))?.end()
    }
}

mod raw {
    use super::*;

    /// This event type is used to indicate new Olm sessions for end-to-end encryption.
    ///
    /// Typically it is encrypted as an *m.room.encrypted* event, then sent as a to-device event.
    ///
    /// The event does not have any content associated with it. The sending client is expected to
    /// send a key share request shortly after this message, causing the receiving client to process
    /// this *m.dummy* event as the most recent event and using the keyshare request to set up the
    /// session. The keyshare request and *m.dummy* combination should result in the original
    /// sending client receiving keys over the newly established session.
    #[derive(Clone, Debug, Deserialize)]
    pub struct DummyEvent {
        /// The event's content.
        pub content: DummyEventContent,
    }

    /// The payload for `DummyEvent`.
    #[derive(Clone, Debug)]
    pub struct DummyEventContent;

    impl<'de> Deserialize<'de> for DummyEventContent {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>
        {
            struct EmptyMapVisitor;

            impl <'de> Visitor<'de> for EmptyMapVisitor {
                type Value = DummyEventContent;

                fn expecting(&self, f: &mut Formatter) -> FmtResult {
                    write!(f, "an object/map")
                }

                fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: MapAccess<'de>
                {
                    Ok(DummyEventContent)
                }
            }

            deserializer.deserialize_map(EmptyMapVisitor)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DummyEvent, DummyEventContent};

    #[test]
    fn serialization()  {
        let dummy_event = DummyEvent {
            content: DummyEventContent,
        };

        let actual = serde_json::to_string(&dummy_event).unwrap();
        let expected = r#"{"content":{},"type":"m.dummy"}"#;

        assert_eq!(actual, expected);
    }

    #[test]
    fn deserialization() {
        let json = r#"{"content":{},"type":"m.dummy"}"#;

        assert!(DummyEvent::from_str(json).is_ok());
    }
}
