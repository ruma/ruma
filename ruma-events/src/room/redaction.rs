//! Types for the *m.room.redaction* event.

use std::time::SystemTime;

use ruma_events_macros::{Event, EventContent};
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{Deserialize, Serialize};

use crate::UnsignedData;

/// Redaction event.
#[derive(Clone, Debug, Event)]
pub struct RedactionEvent {
    /// Data specific to the event type.
    pub content: RedactionEventContent,

    /// The ID of the event that was redacted.
    pub redacts: EventId,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: SystemTime,

    /// The ID of the room associated with this event.
    pub room_id: RoomId,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: UnsignedData,
}

/// Redaction event without a `room_id`.
#[derive(Clone, Debug, Event)]
pub struct RedactionEventStub {
    /// Data specific to the event type.
    pub content: RedactionEventContent,

    /// The ID of the event that was redacted.
    pub redacts: EventId,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: SystemTime,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: UnsignedData,
}

/// A redaction of an event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "m.room.redaction")]
pub struct RedactionEventContent {
    /// The reason for the redaction, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl ruma_events::RoomEventContent for RedactionEventContent {}

impl ruma_events::MessageEventContent for RedactionEventContent {}

#[cfg(test)]
mod tests {
    use std::{
        convert::TryFrom,
        time::{Duration, UNIX_EPOCH},
    };

    use matches::assert_matches;
    use ruma_identifiers::{EventId, RoomId, UserId};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{RedactionEvent, RedactionEventContent};
    use crate::{EventJson, UnsignedData};

    #[test]
    fn serialization() {
        let event = RedactionEvent {
            content: RedactionEventContent { reason: Some("redacted because".into()) },
            redacts: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
            room_id: RoomId::try_from("!roomid:room.com").unwrap(),
            sender: UserId::try_from("@carl:example.com").unwrap(),
            unsigned: UnsignedData::default(),
        };

        let json = json!({
            "content": {
                "reason": "redacted because"
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "redacts": "$h29iv0s8:example.com",
            "room_id": "!roomid:room.com",
            "sender": "@carl:example.com",
            "type": "m.room.redaction",
        });

        assert_eq!(to_json_value(&event).unwrap(), json);
    }

    #[test]
    fn deserialization() {
        let e_id = EventId::try_from("$h29iv0s8:example.com").unwrap();

        let json = json!({
            "content": {
                "reason": "redacted because"
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "redacts": "$h29iv0s8:example.com",
            "room_id": "!roomid:room.com",
            "sender": "@carl:example.com",
            "type": "m.room.redaction",
        });

        assert_matches!(
            from_json_value::<EventJson<RedactionEvent>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            RedactionEvent {
                content: RedactionEventContent {
                    reason: Some(reason),
                },
                sender,
                event_id, origin_server_ts, redacts, room_id, unsigned,
            } if reason == "redacted because" && redacts == e_id
                && event_id == e_id
                && sender == "@carl:example.com"
                && origin_server_ts == UNIX_EPOCH + Duration::from_millis(1)
                && room_id == RoomId::try_from("!roomid:room.com").unwrap()
                && unsigned.is_empty()
        );
    }
}
