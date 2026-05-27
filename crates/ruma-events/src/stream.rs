//! Types for the `m.stream` namespace ([MSC4471]).
//!
//! [MSC4471]: https://github.com/matrix-org/matrix-spec-proposals/pull/4471

use js_int::UInt;
use ruma_common::OwnedDeviceId;
use serde::{Deserialize, Serialize};

pub mod cancel;
pub mod subscribe;
pub mod update;

/// A descriptor advertising that a room event has a live event stream.
///
/// See [MSC4471] for the proposal.
///
/// [MSC4471]: https://github.com/matrix-org/matrix-spec-proposals/pull/4471
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct StreamDescriptor {
    /// The publisher device, owned by the sender of the room event containing the descriptor.
    pub device_id: OwnedDeviceId,

    /// The lifetime of the descriptor in milliseconds, counted from the room
    /// event's `origin_server_ts`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry_ms: Option<UInt>,
}

impl StreamDescriptor {
    /// Creates a new `StreamDescriptor` for the given publisher device.
    pub fn new(device_id: OwnedDeviceId) -> Self {
        Self { device_id, expiry_ms: None }
    }
}

#[cfg(test)]
mod tests {
    use js_int::uint;
    use ruma_common::{canonical_json::assert_to_canonical_json_eq, owned_device_id, serde::Raw};
    use serde_json::json;

    use super::StreamDescriptor;
    use crate::room::message::{RoomMessageEventContent, RoomMessageEventContentWithoutRelation};

    #[test]
    fn descriptor_round_trips_inside_room_message() {
        let mut content = RoomMessageEventContent::text_plain("Hello");
        content.stream = Some(StreamDescriptor {
            device_id: owned_device_id!("DEVICEID"),
            expiry_ms: Some(uint!(1_800_000)),
        });

        assert_to_canonical_json_eq!(
            content,
            json!({
                "msgtype": "m.text",
                "body": "Hello",
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
        let mut content = RoomMessageEventContent::text_plain("Hello");
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
}
