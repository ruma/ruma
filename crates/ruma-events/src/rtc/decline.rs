//! Type for the MatrixRTC decline event ([MSC4310]).
//!
//! Unstable: `org.matrix.msc4310.rtc.decline`
//!
//! This event is sent as a reference relation to an `m.rtc.notification` event.
//!
//! [MSC4310]: https://github.com/matrix-org/matrix-spec-proposals/pull/4310

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
    pub fn new<E: Into<ruma_common::EventId>>(notification_event_id: E) -> Self {
        Self { relates_to: Reference::new(notification_event_id.into()) }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use js_int::uint;
    use ruma_common::{canonical_json::assert_to_canonical_json_eq, owned_event_id};
    use serde_json::{from_value as from_json_value, json};

    use super::RtcDeclineEventContent;
    use crate::{AnyMessageLikeEvent, MessageLikeEvent};

    #[test]
    fn decline_event_serialization() {
        let content = RtcDeclineEventContent::new(owned_event_id!("$abc:example.org"));

        assert_to_canonical_json_eq!(
            content,
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
                "m.relates_to": {
                    "rel_type": "m.reference",
                    "event_id": "$abc:example.org"
                },
            },
            "event_id": "$event:notareal.hs",
            "origin_server_ts": 134_829_848,
            "room_id": "!roomid:notareal.hs",
            "sender": "@user:notareal.hs",
            "type": "m.rtc.decline"
        });

        let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
        assert_matches!(
            event,
            AnyMessageLikeEvent::RtcDecline(MessageLikeEvent::Original(decline_event))
        );
        assert_eq!(decline_event.sender, "@user:notareal.hs");
        assert_eq!(decline_event.origin_server_ts.get(), uint!(134_829_848));
        assert_eq!(decline_event.room_id, "!roomid:notareal.hs");
        assert_eq!(decline_event.content.relates_to.event_id, "$abc:example.org");
    }
}
