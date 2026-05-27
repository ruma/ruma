//! Types for the `m.stream.cancel` to-device event ([MSC4471]).
//!
//! [MSC4471]: https://github.com/matrix-org/matrix-spec-proposals/pull/4471

use ruma_common::{OwnedDeviceId, OwnedEventId, OwnedRoomId, serde::StringEnum};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::PrivOwnedStr;

/// The content of a to-device `m.stream.cancel` event.
///
/// Sent by either side of a subscription to cancel it.
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

    /// A machine-readable cancellation code.
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
///
/// Custom error codes should use the Java package naming convention.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[ruma_enum(rename_all(prefix = "m.", rule = "snake_case"))]
#[non_exhaustive]
pub enum StreamCancelCode {
    /// The publisher has no active stream for the requested descriptor, or the descriptor
    /// has expired.
    UnknownStream,

    /// The subscription request is malformed or names an invalid subscriber device.
    InvalidSubscription,

    /// The subscriber is not allowed to receive updates.
    Forbidden,

    /// The publisher has hit an implementation limit.
    LimitExceeded,

    /// The subscriber no longer wants updates.
    UserCancelled,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::{
        canonical_json::assert_to_canonical_json_eq, owned_device_id, owned_event_id, owned_room_id,
    };
    use serde_json::{from_value as from_json_value, json};

    use super::{StreamCancelCode, ToDeviceStreamCancelEventContent};
    use crate::{AnyToDeviceEvent, ToDeviceEvent};

    #[test]
    fn cancel_round_trip() {
        let mut content = ToDeviceStreamCancelEventContent::new(
            owned_room_id!("!room:example.org"),
            owned_event_id!("$event:example.org"),
            owned_device_id!("SUBSCRIBERDEVICE"),
            StreamCancelCode::UnknownStream,
        );
        content.reason = Some("because".to_owned());

        assert_to_canonical_json_eq!(
            content,
            json!({
                "room_id": "!room:example.org",
                "event_id": "$event:example.org",
                "subscriber_device_id": "SUBSCRIBERDEVICE",
                "code": "m.unknown_stream",
                "reason": "because",
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
    fn cancel_code_deserialization() {
        for (s, expected) in [
            ("m.unknown_stream", StreamCancelCode::UnknownStream),
            ("m.invalid_subscription", StreamCancelCode::InvalidSubscription),
            ("m.forbidden", StreamCancelCode::Forbidden),
            ("m.limit_exceeded", StreamCancelCode::LimitExceeded),
            ("m.user_cancelled", StreamCancelCode::UserCancelled),
        ] {
            let code = from_json_value::<StreamCancelCode>(json!(s)).unwrap();
            assert_eq!(code, expected);
        }
    }

    #[test]
    fn unknown_m_namespace_cancel_code_goes_to_custom() {
        let code = from_json_value::<StreamCancelCode>(json!("m.future_code")).unwrap();
        assert_to_canonical_json_eq!(code, json!("m.future_code"));
        assert_ne!(code, StreamCancelCode::UnknownStream);
        assert_ne!(code, StreamCancelCode::InvalidSubscription);
        assert_ne!(code, StreamCancelCode::Forbidden);
        assert_ne!(code, StreamCancelCode::LimitExceeded);
        assert_ne!(code, StreamCancelCode::UserCancelled);
    }

    #[test]
    fn custom_cancel_code_round_trips() {
        let code = from_json_value::<StreamCancelCode>(json!("io.example.custom_reason")).unwrap();
        assert_to_canonical_json_eq!(code, json!("io.example.custom_reason"));
    }

    #[test]
    fn any_to_device_cancel() {
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

    #[test]
    fn any_to_device_cancel_stable_alias() {
        let event = json!({
            "sender": "@alice:example.org",
            "type": "m.stream.cancel",
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
