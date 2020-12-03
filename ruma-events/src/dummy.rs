//! Types for the *m.dummy* event.

use std::ops::{Deref, DerefMut};

use ruma_events_macros::BasicEventContent;
use ruma_serde::empty::Empty;
use serde::{Deserialize, Serialize};

use crate::BasicEvent;

/// This event type is used to indicate new Olm sessions for end-to-end encryption.
///
/// Typically it is encrypted as an *m.room.encrypted* event, then sent as a to-device event.
///
/// The event does not have any content associated with it. The sending client is expected to
/// send a key share request shortly after this message, causing the receiving client to process
/// this *m.dummy* event as the most recent event and using the keyshare request to set up the
/// session. The keyshare request and *m.dummy* combination should result in the original
/// sending client receiving keys over the newly established session.
pub type DummyEvent = BasicEvent<DummyEventContent>;

/// The payload for `DummyEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, BasicEventContent)]
#[ruma_event(type = "m.dummy")]
pub struct DummyEventContent(pub Empty);

/// The to-device version of the payload for the `DummyEvent`.
pub type DummyToDeviceEventContent = DummyEventContent;

impl Deref for DummyEventContent {
    type Target = Empty;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DummyEventContent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{DummyEvent, DummyEventContent, Empty};
    use ruma_serde::Raw;

    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    #[test]
    fn serialization() {
        let dummy_event = DummyEvent { content: DummyEventContent(Empty) };
        let actual = to_json_value(dummy_event).unwrap();

        let expected = json!({
            "content": {},
            "type": "m.dummy"
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn deserialization() {
        let json = json!({
            "content": {},
            "type": "m.dummy"
        });

        assert!(from_json_value::<Raw<DummyEvent>>(json).unwrap().deserialize().is_ok());
    }
}
