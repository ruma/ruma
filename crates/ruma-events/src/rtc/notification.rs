//! Type for the MatrixRTC notification event (MSC4075).
//!
//! Stable: `m.rtc.notification`
//! Unstable: `org.matrix.msc4075.rtc.notification`

use js_int::UInt;
use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_events::{relation::Reference, Mentions};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of an `m.rtc.notification` event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(
    type = "m.rtc.notification",
    kind = MessageLike
)]
pub struct RtcNotificationEventContent {
    /// Local timestamp observed by the sender device.
    ///
    /// Used with `lifetime` to determine validity; receivers SHOULD compare with
    /// `origin_server_ts` and prefer it if the difference is large.
    pub sender_ts: MilliSecondsSinceUnixEpoch,

    /// Relative time from `sender_ts` during which the notification is considered valid.
    pub lifetime: UInt,

    /// Intentional mentions determining who should be notified.
    #[serde(rename = "m.mentions", default, skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Mentions>,

    /// Optional reference to the related `m.rtc.member` event.
    #[serde(rename = "m.relates_to", skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<Reference>,

    /// How this notification should notify the receiver.
    #[serde(rename = "notification_type")]
    pub notification_type: NotificationType,
}

impl RtcNotificationEventContent {
    /// Creates a new `RtcNotificationEventContent` with the given configuration.
    pub fn new(
        sender_ts: MilliSecondsSinceUnixEpoch,
        lifetime: UInt,
        notification_type: NotificationType,
    ) -> Self {
        Self { sender_ts, lifetime, mentions: None, relates_to: None, notification_type }
    }

    /// Calculates the timestamp at which this notification is considered invalid.
    /// This calculation is based on MSC4075 and tries to use the `sender_ts` as the starting point
    /// and the `lifetime` as the duration for which the notification is valid.
    ///
    /// The `sender_ts` cannot be trusted since it is a generated value by the sending client.
    /// To mitigate issue because of misconfigured client clocks, the MSC requires
    /// that the `origin_server_ts` is used as the starting point if the difference is large.
    ///
    /// # Arguments:
    ///
    /// - `max_sender_ts_offset` is the maximum allowed offset between the two timestamps. (default
    ///   20s)
    /// - `origin_server_ts` has to be set to the origin_server_ts from the event containing this
    ///   event content.
    ///
    /// # Examples
    /// To start a timer until this client should stop ringing for this notification:
    /// ```
    /// let duration_ring = MilliSecondsSinceUnixEpoch::now()
    ///     .saturated_sub(content.invalid_ts(event.origin_server_ts(), None));
    /// ```
    pub fn invalid_ts(
        &self,
        origin_server_ts: MilliSecondsSinceUnixEpoch,
        max_sender_ts_offset: Option<u32>,
    ) -> MilliSecondsSinceUnixEpoch {
        let (larger, smaller) = if self.sender_ts.get() > origin_server_ts.get() {
            (self.sender_ts.get(), origin_server_ts.get())
        } else {
            (origin_server_ts.get(), self.sender_ts.get())
        };
        let use_origin_server_ts =
            larger.saturating_sub(smaller) > max_sender_ts_offset.unwrap_or(20_000).into();
        let start_ts =
            if use_origin_server_ts { origin_server_ts.get() } else { self.sender_ts.get() };
        MilliSecondsSinceUnixEpoch(start_ts.saturating_add(self.lifetime))
    }
}

/// How this notification should notify the receiver.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum NotificationType {
    /// The receiving client should ring with an audible sound.
    #[serde(rename = "ring")]
    Ring,

    /// The receiving client should display a visual notification.
    #[serde(rename = "notification")]
    Notification,
}

#[cfg(test)]
mod tests {
    use js_int::{uint, UInt};
    use ruma_common::{owned_event_id, MilliSecondsSinceUnixEpoch};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{NotificationType, RtcNotificationEventContent};
    use crate::{AnyMessageLikeEvent, Mentions, MessageLikeEvent};

    #[test]
    fn notification_event_serialization() {
        let mut content = RtcNotificationEventContent::new(
            MilliSecondsSinceUnixEpoch(UInt::new(1_752_583_130_365).unwrap()),
            uint!(30_000),
            NotificationType::Ring,
        );
        content.mentions = Some(Mentions::with_room_mention());
        content.relates_to = Some(ruma_events::relation::Reference::new(owned_event_id!("$m:ex")));

        assert_eq!(
            to_json_value(&content).unwrap(),
            json!({
                "sender_ts": 1_752_583_130_365_u64,
                "lifetime": 30_000_u32,
                "m.mentions": {"room": true},
                "m.relates_to": {"rel_type": "m.reference", "event_id": "$m:ex"},
                "notification_type": "ring"
            })
        );
    }

    #[test]
    fn notification_event_deserialization() {
        let json_data = json!({
            "content": {
                "sender_ts": 1_752_583_130_365_u64,
                "lifetime": 30_000_u32,
                "m.mentions": {"room": true},
                "m.relates_to": {"rel_type": "m.reference", "event_id": "$m:ex"},
                "notification_type": "notification"
            },
            "event_id": "$event:notareal.hs",
            "origin_server_ts": 134_829_848,
            "room_id": "!roomid:notareal.hs",
            "sender": "@user:notareal.hs",
            "type": "m.rtc.notification"
        });

        let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
        match event {
            AnyMessageLikeEvent::RtcNotification(MessageLikeEvent::Original(ev)) => {
                assert_eq!(ev.content.lifetime, uint!(30_000));
            }
            _ => panic!("wrong event variant"),
        }
    }

    #[test]
    fn invalid_ts_computation() {
        let content = RtcNotificationEventContent::new(
            MilliSecondsSinceUnixEpoch(UInt::new(100_365).unwrap()),
            uint!(30_000),
            NotificationType::Ring,
        );

        // sender_ts is trustworthy
        let origin_server_ts = MilliSecondsSinceUnixEpoch(UInt::new(120_000).unwrap());
        assert_eq!(
            content.invalid_ts(origin_server_ts, None),
            MilliSecondsSinceUnixEpoch(UInt::new(130_365).unwrap())
        );

        // sender_ts is not trustworthy (sender_ts too small), origin_server_ts is used instead
        let origin_server_ts = MilliSecondsSinceUnixEpoch(UInt::new(200_000).unwrap());
        assert_eq!(
            content.invalid_ts(origin_server_ts, None),
            MilliSecondsSinceUnixEpoch(UInt::new(230_000).unwrap())
        );

        // sender_ts is not trustworthy (sender_ts too large), origin_server_ts is used instead
        let origin_server_ts = MilliSecondsSinceUnixEpoch(UInt::new(50_000).unwrap());
        assert_eq!(
            content.invalid_ts(origin_server_ts, None),
            MilliSecondsSinceUnixEpoch(UInt::new(80_000).unwrap())
        );

        // using a custom max offset (result in origin_server_ts)
        let origin_server_ts = MilliSecondsSinceUnixEpoch(UInt::new(130_200).unwrap());
        assert_eq!(
            content.invalid_ts(origin_server_ts, Some(100)),
            MilliSecondsSinceUnixEpoch(UInt::new(160_200).unwrap())
        );

        // using a custom max offset (result in sender_ts)
        let origin_server_ts = MilliSecondsSinceUnixEpoch(UInt::new(100_300).unwrap());
        assert_eq!(
            content.invalid_ts(origin_server_ts, Some(100)),
            MilliSecondsSinceUnixEpoch(UInt::new(130_365).unwrap())
        );
    }
}
