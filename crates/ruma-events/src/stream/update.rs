//! Types for the `m.stream.update` to-device event ([MSC4471]).
//!
//! After applying a stream update, clients should render the transient body
//! and ignore any current `formatted_body`.
//!
//! [MSC4471]: https://github.com/matrix-org/matrix-spec-proposals/pull/4471

use js_int::UInt;
use ruma_common::{OwnedEventId, OwnedRoomId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of a to-device `m.stream.update` event.
///
/// Sent by the publisher device to a subscriber device.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(
    type = "org.matrix.msc4471.stream.update",
    alias = "m.stream.update",
    kind = ToDevice,
)]
pub struct ToDeviceStreamUpdateEventContent {
    /// The room containing the stream descriptor.
    pub room_id: OwnedRoomId,

    /// The event containing the stream descriptor.
    pub event_id: OwnedEventId,

    /// A monotonically increasing sequence number for this subscriber
    /// device's view of the stream.
    ///
    /// Clients should ignore updates whose `seq` is less than or equal to the
    /// latest sequence number already applied for this stream.
    pub seq: UInt,

    /// The update operation.
    #[serde(flatten)]
    pub operation: StreamUpdateOperation,
}

impl ToDeviceStreamUpdateEventContent {
    /// Creates a new `ToDeviceStreamUpdateEventContent` with the given room,
    /// event, sequence number, and operation.
    pub fn new(
        room_id: OwnedRoomId,
        event_id: OwnedEventId,
        seq: UInt,
        operation: StreamUpdateOperation,
    ) -> Self {
        Self { room_id, event_id, seq, operation }
    }
}

/// A stream update operation.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[serde(tag = "op", content = "content", rename_all = "snake_case")]
pub enum StreamUpdateOperation {
    /// Replace the current body with the payload.
    Replace(StreamUpdateContent),

    /// Append the payload to the current body.
    Append(StreamUpdateContent),
}

/// The payload of a message-like stream update.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct StreamUpdateContent {
    /// Text for the operation.
    pub body: String,
}

impl StreamUpdateContent {
    /// Creates a new `StreamUpdateContent` with the given body.
    pub fn new(body: String) -> Self {
        Self { body }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use js_int::uint;
    use ruma_common::{
        canonical_json::assert_to_canonical_json_eq, owned_event_id, owned_room_id, serde::Raw,
    };
    use serde_json::{from_value as from_json_value, json};

    use super::{StreamUpdateContent, StreamUpdateOperation, ToDeviceStreamUpdateEventContent};
    use crate::{AnyToDeviceEvent, ToDeviceEvent};

    #[test]
    fn replace_update_round_trip() {
        let content = ToDeviceStreamUpdateEventContent::new(
            owned_room_id!("!room:example.org"),
            owned_event_id!("$event:example.org"),
            uint!(1),
            StreamUpdateOperation::Replace(StreamUpdateContent::new("hello".to_owned())),
        );

        assert_to_canonical_json_eq!(
            content,
            json!({
                "room_id": "!room:example.org",
                "event_id": "$event:example.org",
                "seq": 1,
                "op": "replace",
                "content": {
                    "body": "hello",
                },
            })
        );

        let deserialized: ToDeviceStreamUpdateEventContent =
            Raw::new(&content).unwrap().deserialize().unwrap();
        assert_eq!(deserialized.seq, uint!(1));
        assert_matches!(deserialized.operation, StreamUpdateOperation::Replace(payload));
        assert_eq!(payload.body, "hello");
    }

    #[test]
    fn replace_update_seq_zero_round_trip() {
        let content = ToDeviceStreamUpdateEventContent::new(
            owned_room_id!("!room:example.org"),
            owned_event_id!("$event:example.org"),
            uint!(0),
            StreamUpdateOperation::Replace(StreamUpdateContent::new("hello".to_owned())),
        );

        assert_to_canonical_json_eq!(
            content,
            json!({
                "room_id": "!room:example.org",
                "event_id": "$event:example.org",
                "seq": 0,
                "op": "replace",
                "content": {
                    "body": "hello",
                },
            })
        );

        let deserialized: ToDeviceStreamUpdateEventContent =
            Raw::new(&content).unwrap().deserialize().unwrap();
        assert_eq!(deserialized.seq, uint!(0));
        assert_matches!(deserialized.operation, StreamUpdateOperation::Replace(payload));
        assert_eq!(payload.body, "hello");
    }

    #[test]
    fn append_update_round_trip() {
        let content = ToDeviceStreamUpdateEventContent::new(
            owned_room_id!("!room:example.org"),
            owned_event_id!("$event:example.org"),
            uint!(2),
            StreamUpdateOperation::Append(StreamUpdateContent::new(" world".to_owned())),
        );

        assert_to_canonical_json_eq!(
            content,
            json!({
                "room_id": "!room:example.org",
                "event_id": "$event:example.org",
                "seq": 2,
                "op": "append",
                "content": {
                    "body": " world",
                },
            })
        );
    }

    #[test]
    fn any_to_device_update() {
        let event = json!({
            "sender": "@alice:example.org",
            "type": "org.matrix.msc4471.stream.update",
            "content": {
                "room_id": "!room:example.org",
                "event_id": "$event:example.org",
                "seq": 1,
                "op": "replace",
                "content": {
                    "body": "hello",
                },
            },
        });

        let event = from_json_value::<AnyToDeviceEvent>(event).unwrap();
        assert_matches!(event, AnyToDeviceEvent::StreamUpdate(ToDeviceEvent { content, .. }));
        assert_matches!(content.operation, StreamUpdateOperation::Replace(payload));
        assert_eq!(payload.body, "hello");
    }

    #[test]
    fn any_to_device_update_stable_alias() {
        let event = json!({
            "sender": "@alice:example.org",
            "type": "m.stream.update",
            "content": {
                "room_id": "!room:example.org",
                "event_id": "$event:example.org",
                "seq": 1,
                "op": "replace",
                "content": {
                    "body": "hello",
                },
            },
        });

        let event = from_json_value::<AnyToDeviceEvent>(event).unwrap();
        assert_matches!(event, AnyToDeviceEvent::StreamUpdate(ToDeviceEvent { content, .. }));
        assert_matches!(content.operation, StreamUpdateOperation::Replace(payload));
        assert_eq!(payload.body, "hello");
    }
}
