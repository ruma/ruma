//! Types for the [`m.room.redaction`] event.
//!
//! [`m.room.redaction`]: https://spec.matrix.org/v1.2/client-server-api/#mroomredaction

use ruma_macros::{Event, EventContent};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::value::RawValue as RawJsonValue;

use crate::{
    events::{
        EventContent, MessageLikeEventType, MessageLikeUnsigned, Redact, RedactContent,
        RedactedUnsigned, RedactionDeHelper,
    },
    serde::from_raw_json_value,
    EventId, MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId, OwnedUserId, RoomId, UserId,
};

/// A possibly-redacted redaction event.
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum RoomRedactionEvent {
    /// Original, unredacted form of the event.
    Original(OriginalRoomRedactionEvent),

    /// Redacted form of the event with minimal fields.
    Redacted(RedactedRoomRedactionEvent),
}

/// A possibly-redacted redaction event without a `room_id`.
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum SyncRoomRedactionEvent {
    /// Original, unredacted form of the event.
    Original(OriginalSyncRoomRedactionEvent),

    /// Redacted form of the event with minimal fields.
    Redacted(RedactedSyncRoomRedactionEvent),
}

/// Redaction event.
#[derive(Clone, Debug, Event)]
#[allow(clippy::exhaustive_structs)]
pub struct OriginalRoomRedactionEvent {
    /// Data specific to the event type.
    pub content: RoomRedactionEventContent,

    /// The ID of the event that was redacted.
    pub redacts: OwnedEventId,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: OwnedEventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// The ID of the room associated with this event.
    pub room_id: OwnedRoomId,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: MessageLikeUnsigned,
}

impl Redact for OriginalRoomRedactionEvent {
    type Redacted = RedactedRoomRedactionEvent;

    fn redact(
        self,
        redaction: SyncRoomRedactionEvent,
        version: &crate::RoomVersionId,
    ) -> Self::Redacted {
        RedactedRoomRedactionEvent {
            content: self.content.redact(version),
            event_id: self.event_id,
            sender: self.sender,
            origin_server_ts: self.origin_server_ts,
            room_id: self.room_id,
            unsigned: RedactedUnsigned::new_because(Box::new(redaction)),
        }
    }
}

/// Redacted redaction event.
#[derive(Clone, Debug, Event)]
#[allow(clippy::exhaustive_structs)]
pub struct RedactedRoomRedactionEvent {
    /// Data specific to the event type.
    pub content: RedactedRoomRedactionEventContent,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: OwnedEventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// The ID of the room associated with this event.
    pub room_id: OwnedRoomId,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: RedactedUnsigned,
}

/// Redaction event without a `room_id`.
#[derive(Clone, Debug, Event)]
#[allow(clippy::exhaustive_structs)]
pub struct OriginalSyncRoomRedactionEvent {
    /// Data specific to the event type.
    pub content: RoomRedactionEventContent,

    /// The ID of the event that was redacted.
    pub redacts: OwnedEventId,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: OwnedEventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: MessageLikeUnsigned,
}

impl Redact for OriginalSyncRoomRedactionEvent {
    type Redacted = RedactedSyncRoomRedactionEvent;

    fn redact(
        self,
        redaction: SyncRoomRedactionEvent,
        version: &crate::RoomVersionId,
    ) -> Self::Redacted {
        RedactedSyncRoomRedactionEvent {
            content: self.content.redact(version),
            event_id: self.event_id,
            sender: self.sender,
            origin_server_ts: self.origin_server_ts,
            unsigned: RedactedUnsigned::new_because(Box::new(redaction)),
        }
    }
}

/// Redacted redaction event without a `room_id`.
#[derive(Clone, Debug, Event)]
#[allow(clippy::exhaustive_structs)]
pub struct RedactedSyncRoomRedactionEvent {
    /// Data specific to the event type.
    pub content: RedactedRoomRedactionEventContent,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: OwnedEventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: RedactedUnsigned,
}

/// A redaction of an event.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.redaction", kind = MessageLike)]
pub struct RoomRedactionEventContent {
    /// The reason for the redaction, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl RoomRedactionEventContent {
    /// Creates an empty `RoomRedactionEventContent`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `RoomRedactionEventContent` with the given reason.
    pub fn with_reason(reason: String) -> Self {
        Self { reason: Some(reason) }
    }
}

impl RoomRedactionEvent {
    /// Returns the `type` of this event.
    pub fn event_type(&self) -> MessageLikeEventType {
        match self {
            Self::Original(ev) => ev.content.event_type(),
            Self::Redacted(ev) => ev.content.event_type(),
        }
    }

    /// Returns this event's `event_id` field.
    pub fn event_id(&self) -> &EventId {
        match self {
            Self::Original(ev) => &ev.event_id,
            Self::Redacted(ev) => &ev.event_id,
        }
    }

    /// Returns this event's `sender` field.
    pub fn sender(&self) -> &UserId {
        match self {
            Self::Original(ev) => &ev.sender,
            Self::Redacted(ev) => &ev.sender,
        }
    }

    /// Returns this event's `origin_server_ts` field.
    pub fn origin_server_ts(&self) -> MilliSecondsSinceUnixEpoch {
        match self {
            Self::Original(ev) => ev.origin_server_ts,
            Self::Redacted(ev) => ev.origin_server_ts,
        }
    }

    /// Returns this event's `room_id` field.
    pub fn room_id(&self) -> &RoomId {
        match self {
            Self::Original(ev) => &ev.room_id,
            Self::Redacted(ev) => &ev.room_id,
        }
    }

    /// Get the inner `RoomRedactionEvent` if this is an unredacted event.
    pub fn as_original(&self) -> Option<&OriginalRoomRedactionEvent> {
        match self {
            Self::Original(v) => Some(v),
            _ => None,
        }
    }
}

impl Redact for RoomRedactionEvent {
    type Redacted = Self;

    fn redact(self, redaction: SyncRoomRedactionEvent, version: &crate::RoomVersionId) -> Self {
        match self {
            Self::Original(ev) => Self::Redacted(ev.redact(redaction, version)),
            Self::Redacted(ev) => Self::Redacted(ev),
        }
    }
}

impl<'de> Deserialize<'de> for RoomRedactionEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let RedactionDeHelper { unsigned } = from_raw_json_value(&json)?;

        if unsigned.and_then(|u| u.redacted_because).is_some() {
            Ok(Self::Redacted(from_raw_json_value(&json)?))
        } else {
            Ok(Self::Original(from_raw_json_value(&json)?))
        }
    }
}

impl SyncRoomRedactionEvent {
    /// Returns the `type` of this event.
    pub fn event_type(&self) -> MessageLikeEventType {
        match self {
            Self::Original(ev) => ev.content.event_type(),
            Self::Redacted(ev) => ev.content.event_type(),
        }
    }

    /// Returns this event's `event_id` field.
    pub fn event_id(&self) -> &EventId {
        match self {
            Self::Original(ev) => &ev.event_id,
            Self::Redacted(ev) => &ev.event_id,
        }
    }

    /// Returns this event's `sender` field.
    pub fn sender(&self) -> &UserId {
        match self {
            Self::Original(ev) => &ev.sender,
            Self::Redacted(ev) => &ev.sender,
        }
    }

    /// Returns this event's `origin_server_ts` field.
    pub fn origin_server_ts(&self) -> MilliSecondsSinceUnixEpoch {
        match self {
            Self::Original(ev) => ev.origin_server_ts,
            Self::Redacted(ev) => ev.origin_server_ts,
        }
    }

    /// Get the inner `SyncRoomRedactionEvent` if this is an unredacted event.
    pub fn as_original(&self) -> Option<&OriginalSyncRoomRedactionEvent> {
        match self {
            Self::Original(v) => Some(v),
            _ => None,
        }
    }

    /// Convert this sync event into a full event (one with a `room_id` field).
    pub fn into_full_event(self, room_id: OwnedRoomId) -> RoomRedactionEvent {
        match self {
            Self::Original(ev) => RoomRedactionEvent::Original(ev.into_full_event(room_id)),
            Self::Redacted(ev) => RoomRedactionEvent::Redacted(ev.into_full_event(room_id)),
        }
    }
}

impl From<RoomRedactionEvent> for SyncRoomRedactionEvent {
    fn from(full: RoomRedactionEvent) -> Self {
        match full {
            RoomRedactionEvent::Original(ev) => Self::Original(ev.into()),
            RoomRedactionEvent::Redacted(ev) => Self::Redacted(ev.into()),
        }
    }
}

impl Redact for SyncRoomRedactionEvent {
    type Redacted = Self;

    fn redact(self, redaction: SyncRoomRedactionEvent, version: &crate::RoomVersionId) -> Self {
        match self {
            Self::Original(ev) => Self::Redacted(ev.redact(redaction, version)),
            Self::Redacted(ev) => Self::Redacted(ev),
        }
    }
}

impl<'de> Deserialize<'de> for SyncRoomRedactionEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let RedactionDeHelper { unsigned } = from_raw_json_value(&json)?;

        if unsigned.and_then(|u| u.redacted_because).is_some() {
            Ok(Self::Redacted(from_raw_json_value(&json)?))
        } else {
            Ok(Self::Original(from_raw_json_value(&json)?))
        }
    }
}
