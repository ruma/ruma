//! Type for the MatrixRTC decline event (MSC4310).
//!
//! Unstable: `org.matrix.msc4310.rtc.decline`
//!
//! This event is sent as a reference relation to a `m.rtc.notification` event.

use ruma_events::relation::Reference;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of an `m.rtc.decline` event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc4310.rtc.decline", alias = "m.rtc.decline", kind = MessageLike)]
pub struct RtcDeclineEventContent {
    /// The reference to the original call notification message event.
    ///
    /// This must be an `m.reference` to the `m.rtc.notification` / `m.call.notify` event.
    #[serde(rename = "m.relates_to")]
    pub relates_to: Reference,
}

impl RtcDeclineEventContent {
    /// Creates a new `RtcDeclineEventContent` targeting the given notification event id.
    pub fn new<E: Into<ruma_common::OwnedEventId>>(notification_event_id: E) -> Self {
        Self { relates_to: Reference::new(notification_event_id.into()) }
    }
}

#[cfg(test)]
mod tests {
    use js_int::uint;
    use ruma_common::owned_event_id;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::RtcDeclineEventContent;
    use crate::AnyMessageLikeEvent;

    #[test]
    fn decline_event_serialization() {
        let content = RtcDeclineEventContent::new(owned_event_id!("$abc:example.org"));

        let value = to_json_value(&content).unwrap();
        assert_eq!(
            value,
            json!({
                "m.relates_to": {
                    "rel_type": "m.reference",
                    "event_id": "$abc:example.org"
                },
            })
        );
    }

    #[test]
    fn decline_event_deserialization() {
        let json_data = json!({
            "content": {
                "m.relates_to": {"rel_type": "m.reference", "event_id": "$abc:example.org"},
            },
            "event_id": "$event:notareal.hs",
            "origin_server_ts": 134_829_848,
            "room_id": "!roomid:notareal.hs",
            "sender": "@user:notareal.hs",
            "type": "m.rtc.decline"
        });

        let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
        if let AnyMessageLikeEvent::RtcDecline(ce) = event {
            assert_eq!(ce.event_type().to_string(), "org.matrix.msc4310.rtc.decline");
            assert_eq!(ce.origin_server_ts().get(), uint!(134_829_848));
            assert_eq!(ce.room_id().to_string(), "!roomid:notareal.hs");
            assert_eq!(ce.sender().to_string(), "@user:notareal.hs");
            assert_eq!(
                ce.as_original().unwrap().content.relates_to.event_id,
                owned_event_id!("$abc:example.org")
            );
        }
    }
}
