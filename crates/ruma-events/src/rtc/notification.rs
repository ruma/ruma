//! Type for the MatrixRTC notification event ([MSC4075]).
//!
//! Stable: `m.rtc.notification`
//! Unstable: `org.matrix.msc4075.rtc.notification`
//!
//! [MSC4075]: https://github.com/matrix-org/matrix-spec-proposals/pull/4075

use std::time::Duration;

use js_int::UInt;
use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_events::{Mentions, relation::Reference};
use ruma_macros::{EventContent, StringEnum};
use serde::{Deserialize, Serialize};

use crate::PrivOwnedStr;

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
    #[serde(with = "ruma_common::serde::duration::ms")]
    pub lifetime: Duration,

    /// Intentional mentions determining who should be notified.
    #[serde(rename = "m.mentions", default, skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Mentions>,

    /// Optional reference to the related `m.rtc.member` event.
    #[serde(rename = "m.relates_to", skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<Reference>,

    /// How this notification should notify the receiver.
    pub notification_type: NotificationType,

    /// Gives a soft indication of whether the call is a "audio" or "video" (+audio) call.
    /// This is just to indicate between trusted callers that they can start with audio or video
    /// off, but the actual call semantics remain the same, and they may switch at will.
    #[serde(rename = "m.call.intent", skip_serializing_if = "Option::is_none")]
    pub call_intent: Option<CallIntent>,
}

impl RtcNotificationEventContent {
    /// Creates a new `RtcNotificationEventContent` with the given configuration.
    pub fn new(
        sender_ts: MilliSecondsSinceUnixEpoch,
        lifetime: Duration,
        notification_type: NotificationType,
    ) -> Self {
        Self {
            sender_ts,
            lifetime,
            mentions: None,
            relates_to: None,
            notification_type,
            call_intent: None,
        }
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
    /// `let duration_ring =
    /// MilliSecondsSinceUnixEpoch::now().saturated_sub(content.expiration_ts(event.
    /// origin_server_ts(), None));`
    pub fn expiration_ts(
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
        MilliSecondsSinceUnixEpoch(
            start_ts.saturating_add(UInt::from(self.lifetime.as_millis() as u32)),
        )
    }
}

/// How this notification should notify the receiver.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_enum(rename_all = "snake_case")]
pub enum NotificationType {
    /// The receiving client should ring with an audible sound.
    Ring,

    /// The receiving client should display a visual notification.
    Notification,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// Indication of whether the call is a "audio" or "video"(+audio) call.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_enum(rename_all = "snake_case")]
pub enum CallIntent {
    /// Soft indication from the sender that the call is intended for audio.
    Audio,
    /// Soft indication from the sender that the call is intended for video.
    /// Hence that the receiver should start with camera enabled.
    Video,
    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use assert_matches2::assert_matches;
    use js_int::UInt;
    use ruma_common::{
        MilliSecondsSinceUnixEpoch, canonical_json::assert_to_canonical_json_eq, owned_event_id,
    };
    use serde_json::{from_value as from_json_value, json};

    use super::{CallIntent, NotificationType, RtcNotificationEventContent};
    use crate::{AnyMessageLikeEvent, Mentions, MessageLikeEvent};

    #[test]
    fn notification_event_serialization() {
        let mut content = RtcNotificationEventContent::new(
            MilliSecondsSinceUnixEpoch(UInt::new(1_752_583_130_365).unwrap()),
            Duration::from_millis(30_000),
            NotificationType::Ring,
        );
        content.mentions = Some(Mentions::with_room_mention());
        content.relates_to = Some(ruma_events::relation::Reference::new(owned_event_id!("$m:ex")));

        assert_to_canonical_json_eq!(
            content,
            json!({
                "sender_ts": 1_752_583_130_365_u64,
                "lifetime": 30_000_u32,
                "m.mentions": {"room": true},
                "m.relates_to": {"rel_type": "m.reference", "event_id": "$m:ex"},
                "notification_type": "ring",
            })
        );
    }

    #[test]
    fn notification_event_call_intent_serialization() {
        let mut content = RtcNotificationEventContent::new(
            MilliSecondsSinceUnixEpoch(UInt::new(0).unwrap()),
            Duration::from_millis(30_000),
            NotificationType::Notification,
        );
        content.call_intent = Some(CallIntent::Audio);

        assert_to_canonical_json_eq!(
            content,
            json!({
                "sender_ts": 0,
                "lifetime": 30_000_u32,
                "notification_type": "notification",
                "m.call.intent": "audio",
            })
        );
    }

    #[test]
    fn call_intent_deserialization_default() {
        let raw_content = json!({
            "m.mentions": {
                "user_ids": [],
                "room": true
            },
            "notification_type": "ring",
            "m.relates_to": {
                "event_id": "$IACrEkEKgDa-n4cMk-lEJ3vqLLUL9zX1nVyAnpmFaec",
                "rel_type": "m.reference"
            },
            "sender_ts": 17709890710_u64,
            "lifetime": 30000,
        });
        let content: RtcNotificationEventContent = from_json_value(raw_content).unwrap();
        assert_eq!(content.call_intent, None);
    }

    #[test]
    fn test_call_intent_serialization() {
        assert_eq!(serde_json::to_string(&CallIntent::Audio).unwrap(), r#""audio""#);
        assert_eq!(serde_json::to_string(&CallIntent::Video).unwrap(), r#""video""#);
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
        assert_matches!(
            event,
            AnyMessageLikeEvent::RtcNotification(MessageLikeEvent::Original(ev))
        );
        assert_eq!(ev.content.lifetime, Duration::from_millis(30_000));
    }

    #[test]
    fn expiration_ts_computation() {
        let content = RtcNotificationEventContent::new(
            MilliSecondsSinceUnixEpoch(UInt::new(100_365).unwrap()),
            Duration::from_millis(30_000),
            NotificationType::Ring,
        );

        // sender_ts is trustworthy
        let origin_server_ts = MilliSecondsSinceUnixEpoch(UInt::new(120_000).unwrap());
        assert_eq!(
            content.expiration_ts(origin_server_ts, None),
            MilliSecondsSinceUnixEpoch(UInt::new(130_365).unwrap())
        );

        // sender_ts is not trustworthy (sender_ts too small), origin_server_ts is used instead
        let origin_server_ts = MilliSecondsSinceUnixEpoch(UInt::new(200_000).unwrap());
        assert_eq!(
            content.expiration_ts(origin_server_ts, None),
            MilliSecondsSinceUnixEpoch(UInt::new(230_000).unwrap())
        );

        // sender_ts is not trustworthy (sender_ts too large), origin_server_ts is used instead
        let origin_server_ts = MilliSecondsSinceUnixEpoch(UInt::new(50_000).unwrap());
        assert_eq!(
            content.expiration_ts(origin_server_ts, None),
            MilliSecondsSinceUnixEpoch(UInt::new(80_000).unwrap())
        );

        // using a custom max offset (result in origin_server_ts)
        let origin_server_ts = MilliSecondsSinceUnixEpoch(UInt::new(130_200).unwrap());
        assert_eq!(
            content.expiration_ts(origin_server_ts, Some(100)),
            MilliSecondsSinceUnixEpoch(UInt::new(160_200).unwrap())
        );

        // using a custom max offset (result in sender_ts)
        let origin_server_ts = MilliSecondsSinceUnixEpoch(UInt::new(100_300).unwrap());
        assert_eq!(
            content.expiration_ts(origin_server_ts, Some(100)),
            MilliSecondsSinceUnixEpoch(UInt::new(130_365).unwrap())
        );
    }
}
