//! Types for [MSC4471] event streams.
//!
//! An event stream is a live, non-durable companion to a room event. Clients
//! display incremental updates while the event is changing and discard them
//! once the final content is committed to the room.
//!
//! [MSC4471]: https://github.com/matrix-org/matrix-spec-proposals/pull/4471

use js_int::UInt;
use ruma_common::{OwnedDeviceId, OwnedEventId, OwnedRoomId, serde::StringEnum};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::PrivOwnedStr;

/// A descriptor advertising that a room event has a live event stream.
///
/// This uses the unstable prefix defined in [MSC4471].
///
/// [MSC4471]: https://github.com/matrix-org/matrix-spec-proposals/pull/4471
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct StreamDescriptor {
    /// The publisher device to which subscriptions should be sent.
    ///
    /// This device must be owned by the sender of the room event containing
    /// the descriptor.
    pub device_id: OwnedDeviceId,

    /// The lifetime of the descriptor in milliseconds, counted from the room
    /// event's `origin_server_ts`.
    ///
    /// If omitted, clients should assume a short implementation-defined
    /// lifetime.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry_ms: Option<UInt>,
}

impl StreamDescriptor {
    /// Creates a new `StreamDescriptor` for the given publisher device.
    pub fn new(device_id: OwnedDeviceId) -> Self {
        Self { device_id, expiry_ms: None }
    }
}

/// The content of a to-device `m.stream.subscribe` event.
///
/// Sent by a subscriber device to the publisher device named in a stream
/// descriptor to request updates for `(room_id, event_id)`.
///
/// This uses the unstable prefix defined in [MSC4471].
///
/// [MSC4471]: https://github.com/matrix-org/matrix-spec-proposals/pull/4471
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(
    type = "org.matrix.msc4471.stream.subscribe",
    alias = "m.stream.subscribe",
    kind = ToDevice,
)]
pub struct ToDeviceStreamSubscribeEventContent {
    /// The room containing the stream descriptor.
    pub room_id: OwnedRoomId,

    /// The event containing the stream descriptor.
    pub event_id: OwnedEventId,

    /// The subscriber device which should receive updates.
    pub subscriber_device_id: OwnedDeviceId,

    /// If `true`, the subscriber device requests a fresh `replace` baseline.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    pub resync: bool,
}

impl ToDeviceStreamSubscribeEventContent {
    /// Creates a new `ToDeviceStreamSubscribeEventContent` with the given
    /// room, event, and subscriber device.
    pub fn new(
        room_id: OwnedRoomId,
        event_id: OwnedEventId,
        subscriber_device_id: OwnedDeviceId,
    ) -> Self {
        Self { room_id, event_id, subscriber_device_id, resync: false }
    }
}

/// The content of a to-device `m.stream.cancel` event.
///
/// Sent by either side of a subscription to cancel it.
///
/// This uses the unstable prefix defined in [MSC4471].
///
/// [MSC4471]: https://github.com/matrix-org/matrix-spec-proposals/pull/4471
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(
    type = "org.matrix.msc4471.stream.cancel",
    alias = "m.stream.cancel",
    kind = ToDevice,
)]
pub struct ToDeviceStreamCancelEventContent {
    /// The room containing the stream descriptor.
    pub room_id: OwnedRoomId,

    /// The event containing the stream descriptor.
    pub event_id: OwnedEventId,

    /// The subscriber device whose subscription is cancelled.
    pub subscriber_device_id: OwnedDeviceId,

    /// A machine-readable reason for the cancellation.
    pub code: StreamCancelCode,

    /// A human-readable reason for debugging.
    ///
    /// Clients should not rely on this value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl ToDeviceStreamCancelEventContent {
    /// Creates a new `ToDeviceStreamCancelEventContent` with the given room,
    /// event, subscriber device, and code.
    pub fn new(
        room_id: OwnedRoomId,
        event_id: OwnedEventId,
        subscriber_device_id: OwnedDeviceId,
        code: StreamCancelCode,
    ) -> Self {
        Self { room_id, event_id, subscriber_device_id, code, reason: None }
    }
}

/// A machine-readable cancellation code for an event stream subscription.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[ruma_enum(rename_all(prefix = "m.", rule = "snake_case"))]
#[non_exhaustive]
pub enum StreamCancelCode {
    /// The publisher device does not have an active stream for the requested
    /// descriptor, or the descriptor has expired.
    UnknownStream,

    /// The subscription request is malformed or names an invalid subscriber
    /// device.
    InvalidSubscription,

    /// The publisher device declined because the subscriber is not allowed to
    /// receive updates.
    Forbidden,

    /// The publisher device declined because of implementation limits.
    LimitExceeded,

    /// The subscriber device no longer wants updates for the stream.
    UserCancelled,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The content of a to-device `m.stream.update` event for a message-like
/// stream.
///
/// Sent by the publisher device to a subscriber device. Each update replaces
/// or appends to the transient `body` being rendered for the descriptor
/// event.
///
/// This uses the unstable prefix defined in [MSC4471].
///
/// [MSC4471]: https://github.com/matrix-org/matrix-spec-proposals/pull/4471
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

/// A stream update operation for a message-like stream.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[serde(tag = "op", content = "content", rename_all = "snake_case")]
pub enum StreamUpdateOperation {
    /// Replace the current transient `body` with the given payload.
    Replace(StreamUpdateContent),

    /// Append the given payload's `body` to the current transient `body`.
    Append(StreamUpdateContent),
}

/// The payload of a message-like stream update.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct StreamUpdateContent {
    /// Text to replace or append to the current transient `body`.
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
        canonical_json::assert_to_canonical_json_eq, owned_device_id, owned_event_id,
        owned_room_id, serde::Raw,
    };
    use serde_json::{from_value as from_json_value, json};

    use super::{
        StreamCancelCode, StreamDescriptor, StreamUpdateContent, StreamUpdateOperation,
        ToDeviceStreamCancelEventContent, ToDeviceStreamSubscribeEventContent,
        ToDeviceStreamUpdateEventContent,
    };
    use crate::{
        AnyToDeviceEvent, ToDeviceEvent,
        room::message::{RoomMessageEventContent, RoomMessageEventContentWithoutRelation},
    };

    #[test]
    fn descriptor_round_trips_inside_room_message() {
        let mut content = RoomMessageEventContent::text_plain("Generating response...");
        content.stream = Some(StreamDescriptor {
            device_id: owned_device_id!("DEVICEID"),
            expiry_ms: Some(uint!(1_800_000)),
        });

        assert_to_canonical_json_eq!(
            content,
            json!({
                "msgtype": "m.text",
                "body": "Generating response...",
                "org.matrix.msc4471.stream": {
                    "device_id": "DEVICEID",
                    "expiry_ms": 1_800_000,
                },
            })
        );

        let raw = Raw::new(&content).unwrap();
        let deserialized: RoomMessageEventContent = raw.deserialize().unwrap();
        let stream = deserialized.stream.unwrap();
        assert_eq!(stream.device_id, "DEVICEID");
        assert_eq!(stream.expiry_ms, Some(uint!(1_800_000)));
    }

    #[test]
    fn replacement_strips_descriptor() {
        let mut content = RoomMessageEventContent::text_plain("Generating response...");
        content.stream = Some(StreamDescriptor::new(owned_device_id!("DEVICEID")));

        content.apply_replacement(RoomMessageEventContentWithoutRelation::text_plain("Done"));

        assert!(content.stream.is_none());
        assert_to_canonical_json_eq!(
            content,
            json!({
                "msgtype": "m.text",
                "body": "Done",
            })
        );
    }

    #[test]
    fn subscribe_round_trip() {
        let mut content = ToDeviceStreamSubscribeEventContent::new(
            owned_room_id!("!room:example.org"),
            owned_event_id!("$event:example.org"),
            owned_device_id!("SUBSCRIBERDEVICE"),
        );
        content.resync = true;

        assert_to_canonical_json_eq!(
            content,
            json!({
                "room_id": "!room:example.org",
                "event_id": "$event:example.org",
                "subscriber_device_id": "SUBSCRIBERDEVICE",
                "resync": true,
            })
        );

        let deserialized: ToDeviceStreamSubscribeEventContent =
            Raw::new(&content).unwrap().deserialize().unwrap();
        assert_eq!(deserialized.subscriber_device_id, "SUBSCRIBERDEVICE");
        assert!(deserialized.resync);
    }

    #[test]
    fn subscribe_resync_defaults_to_false() {
        let content = ToDeviceStreamSubscribeEventContent::new(
            owned_room_id!("!room:example.org"),
            owned_event_id!("$event:example.org"),
            owned_device_id!("SUBSCRIBERDEVICE"),
        );

        assert_to_canonical_json_eq!(
            content,
            json!({
                "room_id": "!room:example.org",
                "event_id": "$event:example.org",
                "subscriber_device_id": "SUBSCRIBERDEVICE",
            })
        );
    }

    #[test]
    fn cancel_round_trip() {
        let mut content = ToDeviceStreamCancelEventContent::new(
            owned_room_id!("!room:example.org"),
            owned_event_id!("$event:example.org"),
            owned_device_id!("SUBSCRIBERDEVICE"),
            StreamCancelCode::UnknownStream,
        );
        content.reason = Some("Unknown or expired stream".to_owned());

        assert_to_canonical_json_eq!(
            content,
            json!({
                "room_id": "!room:example.org",
                "event_id": "$event:example.org",
                "subscriber_device_id": "SUBSCRIBERDEVICE",
                "code": "m.unknown_stream",
                "reason": "Unknown or expired stream",
            })
        );
    }

    #[test]
    fn cancel_code_serialization() {
        for (variant, expected) in [
            (StreamCancelCode::UnknownStream, "m.unknown_stream"),
            (StreamCancelCode::InvalidSubscription, "m.invalid_subscription"),
            (StreamCancelCode::Forbidden, "m.forbidden"),
            (StreamCancelCode::LimitExceeded, "m.limit_exceeded"),
            (StreamCancelCode::UserCancelled, "m.user_cancelled"),
        ] {
            assert_to_canonical_json_eq!(variant, json!(expected));
        }
    }

    #[test]
    fn custom_cancel_code_round_trips() {
        let code = from_json_value::<StreamCancelCode>(json!("io.example.custom_reason")).unwrap();
        assert_to_canonical_json_eq!(code, json!("io.example.custom_reason"));
    }

    #[test]
    fn replace_update_round_trip() {
        let content = ToDeviceStreamUpdateEventContent::new(
            owned_room_id!("!room:example.org"),
            owned_event_id!("$event:example.org"),
            uint!(1),
            StreamUpdateOperation::Replace(StreamUpdateContent::new(
                "The answer is still being generated.".to_owned(),
            )),
        );

        assert_to_canonical_json_eq!(
            content,
            json!({
                "room_id": "!room:example.org",
                "event_id": "$event:example.org",
                "seq": 1,
                "op": "replace",
                "content": {
                    "body": "The answer is still being generated.",
                },
            })
        );

        let deserialized: ToDeviceStreamUpdateEventContent =
            Raw::new(&content).unwrap().deserialize().unwrap();
        assert_eq!(deserialized.seq, uint!(1));
        assert_matches!(deserialized.operation, StreamUpdateOperation::Replace(payload));
        assert_eq!(payload.body, "The answer is still being generated.");
    }

    #[test]
    fn append_update_round_trip() {
        let content = ToDeviceStreamUpdateEventContent::new(
            owned_room_id!("!room:example.org"),
            owned_event_id!("$event:example.org"),
            uint!(2),
            StreamUpdateOperation::Append(StreamUpdateContent::new(" Still working.".to_owned())),
        );

        assert_to_canonical_json_eq!(
            content,
            json!({
                "room_id": "!room:example.org",
                "event_id": "$event:example.org",
                "seq": 2,
                "op": "append",
                "content": {
                    "body": " Still working.",
                },
            })
        );
    }

    #[test]
    fn to_device_event_enum_routes_stream_update() {
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
    fn to_device_event_enum_routes_stream_subscribe() {
        let event = json!({
            "sender": "@alice:example.org",
            "type": "org.matrix.msc4471.stream.subscribe",
            "content": {
                "room_id": "!room:example.org",
                "event_id": "$event:example.org",
                "subscriber_device_id": "SUBSCRIBERDEVICE",
            },
        });

        let event = from_json_value::<AnyToDeviceEvent>(event).unwrap();
        assert_matches!(event, AnyToDeviceEvent::StreamSubscribe(ToDeviceEvent { content, .. }));
        assert!(!content.resync);
    }

    #[test]
    fn to_device_event_enum_routes_stream_cancel() {
        let event = json!({
            "sender": "@alice:example.org",
            "type": "org.matrix.msc4471.stream.cancel",
            "content": {
                "room_id": "!room:example.org",
                "event_id": "$event:example.org",
                "subscriber_device_id": "SUBSCRIBERDEVICE",
                "code": "m.user_cancelled",
            },
        });

        let event = from_json_value::<AnyToDeviceEvent>(event).unwrap();
        assert_matches!(event, AnyToDeviceEvent::StreamCancel(ToDeviceEvent { content, .. }));
        assert_eq!(content.code, StreamCancelCode::UserCancelled);
    }
}
