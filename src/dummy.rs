//! Types for the *m.dummy* event.

use ruma_events_macros::ruma_event;

use crate::Empty;

ruma_event! {
    /// This event type is used to indicate new Olm sessions for end-to-end encryption.
    ///
    /// Typically it is encrypted as an *m.room.encrypted* event, then sent as a to-device event.
    ///
    /// The event does not have any content associated with it. The sending client is expected to
    /// send a key share request shortly after this message, causing the receiving client to process
    /// this *m.dummy* event as the most recent event and using the keyshare request to set up the
    /// session. The keyshare request and *m.dummy* combination should result in the original
    /// sending client receiving keys over the newly established session.
    DummyEvent {
        kind: Event,
        event_type: "m.dummy",
        content_type_alias: {
            /// The payload for `DummyEvent`.
            Empty
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DummyEvent, Empty};
    use crate::EventResult;

    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    #[test]
    fn serialization() {
        let dummy_event = DummyEvent { content: Empty };
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

        assert!(from_json_value::<EventResult<DummyEvent>>(json)
            .unwrap()
            .into_result()
            .is_ok());
    }
}
