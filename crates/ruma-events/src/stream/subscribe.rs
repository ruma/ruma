//! Types for the `m.stream.subscribe` to-device event ([MSC4471]).
//!
//! [MSC4471]: https://github.com/matrix-org/matrix-spec-proposals/pull/4471

use ruma_common::{OwnedDeviceId, OwnedEventId, OwnedRoomId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of a to-device `m.stream.subscribe` event.
///
/// Sent by a subscriber device to the publisher device named in a stream descriptor.
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
    ///
    /// The device must belong to the subscribing user; the publisher verifies
    /// this before accepting the subscription.
    pub subscriber_device_id: OwnedDeviceId,

    /// If `true`, request a fresh `replace` operation rather than continuing from the current
    /// state.
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

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::{
        canonical_json::assert_to_canonical_json_eq, owned_device_id, owned_event_id,
        owned_room_id, serde::Raw,
    };
    use serde_json::{from_value as from_json_value, json};

    use super::ToDeviceStreamSubscribeEventContent;
    use crate::{AnyToDeviceEvent, ToDeviceEvent};

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
    fn subscribe_resync_default() {
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
    fn subscribe_resync_default_deserializes_false() {
        let json = json!({
            "room_id": "!room:example.org",
            "event_id": "$event:example.org",
            "subscriber_device_id": "SUBSCRIBERDEVICE",
        });

        let content = from_json_value::<ToDeviceStreamSubscribeEventContent>(json).unwrap();
        assert!(!content.resync);
    }

    #[test]
    fn any_to_device_subscribe() {
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
    fn any_to_device_subscribe_stable_alias() {
        let event = json!({
            "sender": "@alice:example.org",
            "type": "m.stream.subscribe",
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
}
