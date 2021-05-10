//! Types for the *m.dummy* event.

use ruma_events_macros::BasicEventContent;
use serde::{Deserialize, Serialize};

use crate::BasicToDeviceEvent;

/// This event type is used to indicate new Olm sessions for end-to-end encryption.
///
/// Typically it is encrypted as an *m.room.encrypted* event, then sent as a to-device event.
///
/// The event does not have any content associated with it. The sending client is expected to
/// send a key share request shortly after this message, causing the receiving client to process
/// this *m.dummy* event as the most recent event and using the keyshare request to set up the
/// session. The keyshare request and *m.dummy* combination should result in the original
/// sending client receiving keys over the newly established session.
pub type DummyEvent = BasicToDeviceEvent<DummyEventContent>;

/// The payload for `DummyEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, BasicEventContent)]
#[ruma_event(type = "m.dummy")]
pub struct DummyEventContent {}

/// The to-device version of the payload for the `DummyEvent`.
pub type DummyToDeviceEventContent = DummyEventContent;

#[cfg(test)]
mod tests {
    use ruma_serde::Raw;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{DummyEvent, DummyEventContent};

    #[test]
    fn serialization() {
        let dummy_event = DummyEvent { content: DummyEventContent {} };
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
