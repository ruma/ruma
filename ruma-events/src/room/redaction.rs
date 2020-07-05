//! Types for the *m.room.redaction* event.

use std::{collections::BTreeMap, time::SystemTime};

use ruma_events_macros::{Event, EventContent};
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{Deserialize, Serialize};
use serde_json::{value::RawValue as RawJsonValue, Value as JsonValue};

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

/// The content of any event that has been redacted.
///
/// This does not represent the redaction event itself but, the removal of
/// some events content via redaction.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RedactedContent {
    /// The redacted events type.
    #[serde(rename = "type", skip_serializing)]
    pub event_type: String,

    /// The reason for the redaction, if any.
    ///
    /// This field is copied from the redaction event that affected this event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    // TODO how do we want to handle this...
    /// The keys that are allowed to be kept inside of the `content` field
    /// from the original event.
    #[serde(flatten)]
    pub left_over_keys: BTreeMap<String, JsonValue>,
}

#[derive(Deserialize)]
struct RedactHelper {
    reason: Option<String>,

    // This allows the rest of the keys to be collected here.
    #[serde(flatten)]
    left_over_keys: BTreeMap<String, JsonValue>,
}

impl ruma_events::EventContent for RedactedContent {
    fn event_type(&self) -> &str {
        &self.event_type
    }

    fn from_parts(event_type: &str, content: Box<RawJsonValue>) -> Result<Self, serde_json::Error> {
        let RedactHelper { reason, left_over_keys } = serde_json::from_str(content.get())?;
        Ok(Self { event_type: event_type.to_string(), reason, left_over_keys })
    }
}

impl ruma_events::RoomEventContent for RedactedContent {}

impl ruma_events::BasicEventContent for RedactedContent {}

impl ruma_events::MessageEventContent for RedactedContent {}

impl ruma_events::StateEventContent for RedactedContent {}

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
