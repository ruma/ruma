//! Types for the [`m.room.redaction`] event.
//!
//! [`m.room.redaction`]: https://spec.matrix.org/latest/client-server-api/#mroomredaction

use as_variant::as_variant;
use js_int::Int;
#[cfg(feature = "canonical-json")]
use ruma_common::canonical_json::RedactionEvent;
use ruma_common::{
    serde::CanBeEmpty, EventId, MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId,
    OwnedTransactionId, OwnedUserId, RoomId, RoomVersionId, UserId,
};
use ruma_macros::{Event, EventContent};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    BundledMessageLikeRelations, EventContent, MessageLikeEventType, RedactContent,
    RedactedMessageLikeEventContent, RedactedUnsigned, StaticEventContent,
};

mod event_serde;

/// A possibly-redacted redaction event.
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Debug)]
pub enum RoomRedactionEvent {
    /// Original, unredacted form of the event.
    Original(OriginalRoomRedactionEvent),

    /// Redacted form of the event with minimal fields.
    Redacted(RedactedRoomRedactionEvent),
}

/// A possibly-redacted redaction event without a `room_id`.
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Debug)]
pub enum SyncRoomRedactionEvent {
    /// Original, unredacted form of the event.
    Original(OriginalSyncRoomRedactionEvent),

    /// Redacted form of the event with minimal fields.
    Redacted(RedactedSyncRoomRedactionEvent),
}

/// Redaction event.
#[derive(Clone, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct OriginalRoomRedactionEvent {
    /// Data specific to the event type.
    pub content: RoomRedactionEventContent,

    /// The ID of the event that was redacted.
    ///
    /// This field is required in room versions prior to 11.
    pub redacts: Option<OwnedEventId>,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: OwnedEventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// The ID of the room associated with this event.
    pub room_id: OwnedRoomId,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: RoomRedactionUnsigned,
}

impl From<OriginalRoomRedactionEvent> for OriginalSyncRoomRedactionEvent {
    fn from(value: OriginalRoomRedactionEvent) -> Self {
        let OriginalRoomRedactionEvent {
            content,
            redacts,
            event_id,
            sender,
            origin_server_ts,
            unsigned,
            ..
        } = value;

        Self { content, redacts, event_id, sender, origin_server_ts, unsigned }
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
#[derive(Clone, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct OriginalSyncRoomRedactionEvent {
    /// Data specific to the event type.
    pub content: RoomRedactionEventContent,

    /// The ID of the event that was redacted.
    ///
    /// This field is required in room versions prior to 11.
    pub redacts: Option<OwnedEventId>,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: OwnedEventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: RoomRedactionUnsigned,
}

impl OriginalSyncRoomRedactionEvent {
    /// Convert this sync event into a full event, one with a `room_id` field.
    pub fn into_full_event(self, room_id: OwnedRoomId) -> OriginalRoomRedactionEvent {
        let Self { content, redacts, event_id, sender, origin_server_ts, unsigned } = self;

        OriginalRoomRedactionEvent {
            content,
            redacts,
            event_id,
            sender,
            origin_server_ts,
            room_id,
            unsigned,
        }
    }

    pub(crate) fn into_maybe_redacted(self) -> SyncRoomRedactionEvent {
        SyncRoomRedactionEvent::Original(self)
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
#[ruma_event(type = "m.room.redaction", kind = MessageLike, custom_redacted)]
pub struct RoomRedactionEventContent {
    /// The ID of the event that was redacted.
    ///
    /// This field is required starting from room version 11.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redacts: Option<OwnedEventId>,

    /// The reason for the redaction, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl RoomRedactionEventContent {
    /// Creates an empty `RoomRedactionEventContent` according to room versions 1 through 10.
    pub fn new_v1() -> Self {
        Self::default()
    }

    /// Creates a `RoomRedactionEventContent` with the required `redacts` field introduced in room
    /// version 11.
    pub fn new_v11(redacts: OwnedEventId) -> Self {
        Self { redacts: Some(redacts), ..Default::default() }
    }

    /// Add the given reason to this `RoomRedactionEventContent`.
    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }
}

impl RedactContent for RoomRedactionEventContent {
    type Redacted = RedactedRoomRedactionEventContent;

    fn redact(self, version: &RoomVersionId) -> Self::Redacted {
        let redacts = match version {
            RoomVersionId::V1
            | RoomVersionId::V2
            | RoomVersionId::V3
            | RoomVersionId::V4
            | RoomVersionId::V5
            | RoomVersionId::V6
            | RoomVersionId::V7
            | RoomVersionId::V8
            | RoomVersionId::V9
            | RoomVersionId::V10 => None,
            _ => self.redacts,
        };

        RedactedRoomRedactionEventContent { redacts }
    }
}

/// A redacted redaction event.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RedactedRoomRedactionEventContent {
    /// The ID of the event that was redacted.
    ///
    /// This field is required starting from room version 11.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redacts: Option<OwnedEventId>,
}

impl EventContent for RedactedRoomRedactionEventContent {
    type EventType = MessageLikeEventType;

    fn event_type(&self) -> Self::EventType {
        MessageLikeEventType::RoomRedaction
    }
}

impl StaticEventContent for RedactedRoomRedactionEventContent {
    const TYPE: &'static str = "m.room.redaction";
}

impl RedactedMessageLikeEventContent for RedactedRoomRedactionEventContent {}

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

    /// Returns the ID of the event that this event redacts, according to the given room version.
    ///
    /// # Panics
    ///
    /// Panics if this is a non-redacted event and both `redacts` field are `None`, which is only
    /// possible if the event was modified after being deserialized.
    pub fn redacts(&self, room_version: &RoomVersionId) -> Option<&EventId> {
        match self {
            Self::Original(ev) => Some(ev.redacts(room_version)),
            Self::Redacted(ev) => ev.content.redacts.as_deref(),
        }
    }

    /// Get the inner `RoomRedactionEvent` if this is an unredacted event.
    pub fn as_original(&self) -> Option<&OriginalRoomRedactionEvent> {
        as_variant!(self, Self::Original)
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

    /// Returns the ID of the event that this event redacts, according to the given room version.
    ///
    /// # Panics
    ///
    /// Panics if this is a non-redacted event and both `redacts` field are `None`, which is only
    /// possible if the event was modified after being deserialized.
    pub fn redacts(&self, room_version: &RoomVersionId) -> Option<&EventId> {
        match self {
            Self::Original(ev) => Some(ev.redacts(room_version)),
            Self::Redacted(ev) => ev.content.redacts.as_deref(),
        }
    }

    /// Get the inner `SyncRoomRedactionEvent` if this is an unredacted event.
    pub fn as_original(&self) -> Option<&OriginalSyncRoomRedactionEvent> {
        as_variant!(self, Self::Original)
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

impl OriginalRoomRedactionEvent {
    /// Returns the ID of the event that this event redacts, according to the proper `redacts` field
    /// for the given room version.
    ///
    /// If the `redacts` field is not the proper one for the given room version, this falls back to
    /// the one that is available.
    ///
    /// # Panics
    ///
    /// Panics if both `redacts` field are `None`, which is only possible if the event was modified
    /// after being deserialized.
    pub fn redacts(&self, room_version: &RoomVersionId) -> &EventId {
        redacts(room_version, self.redacts.as_deref(), self.content.redacts.as_deref())
    }
}

impl OriginalSyncRoomRedactionEvent {
    /// Returns the ID of the event that this event redacts, according to the proper `redacts` field
    /// for the given room version.
    ///
    /// If the `redacts` field is not the proper one for the given room version, this falls back to
    /// the one that is available.
    ///
    /// # Panics
    ///
    /// Panics if both `redacts` field are `None`, which is only possible if the event was modified
    /// after being deserialized.
    pub fn redacts(&self, room_version: &RoomVersionId) -> &EventId {
        redacts(room_version, self.redacts.as_deref(), self.content.redacts.as_deref())
    }
}

#[cfg(feature = "canonical-json")]
impl RedactionEvent for OriginalRoomRedactionEvent {}
#[cfg(feature = "canonical-json")]
impl RedactionEvent for OriginalSyncRoomRedactionEvent {}
#[cfg(feature = "canonical-json")]
impl RedactionEvent for RoomRedactionEvent {}
#[cfg(feature = "canonical-json")]
impl RedactionEvent for SyncRoomRedactionEvent {}

/// Extra information about a redaction that is not incorporated into the event's hash.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RoomRedactionUnsigned {
    /// The time in milliseconds that has elapsed since the event was sent.
    ///
    /// This field is generated by the local homeserver, and may be incorrect if the local time on
    /// at least one of the two servers is out of sync, which can cause the age to either be
    /// negative or greater than it actually is.
    pub age: Option<Int>,

    /// The client-supplied transaction ID, if the client being given the event is the same one
    /// which sent it.
    pub transaction_id: Option<OwnedTransactionId>,

    /// [Bundled aggregations] of related child events.
    ///
    /// [Bundled aggregations]: https://spec.matrix.org/latest/client-server-api/#aggregations-of-child-events
    #[serde(rename = "m.relations", default)]
    pub relations: BundledMessageLikeRelations<OriginalSyncRoomRedactionEvent>,
}

impl RoomRedactionUnsigned {
    /// Create a new `Unsigned` with fields set to `None`.
    pub fn new() -> Self {
        Self { age: None, transaction_id: None, relations: BundledMessageLikeRelations::default() }
    }
}

impl Default for RoomRedactionUnsigned {
    fn default() -> Self {
        Self::new()
    }
}

impl CanBeEmpty for RoomRedactionUnsigned {
    /// Whether this unsigned data is empty (all fields are `None`).
    ///
    /// This method is used to determine whether to skip serializing the `unsigned` field in room
    /// events. Do not use it to determine whether an incoming `unsigned` field was present - it
    /// could still have been present but contained none of the known fields.
    fn is_empty(&self) -> bool {
        self.age.is_none() && self.transaction_id.is_none() && self.relations.is_empty()
    }
}

/// Returns the value of the proper `redacts` field for the given room version.
///
/// If the `redacts` field is not the proper one for the given room version, this falls back to
/// the one that is available.
///
/// # Panics
///
/// Panics if both `redacts` and `content_redacts` are `None`.
fn redacts<'a>(
    room_version: &'_ RoomVersionId,
    redacts: Option<&'a EventId>,
    content_redacts: Option<&'a EventId>,
) -> &'a EventId {
    match room_version {
            RoomVersionId::V1
            | RoomVersionId::V2
            | RoomVersionId::V3
            | RoomVersionId::V4
            | RoomVersionId::V5
            | RoomVersionId::V6
            | RoomVersionId::V7
            | RoomVersionId::V8
            | RoomVersionId::V9
            | RoomVersionId::V10 => redacts
                .or_else(|| {
                    error!("Redacts field at event level not available, falling back to the one inside content");
                    content_redacts
        })
                .expect("At least one redacts field is set"),
            _ => content_redacts
                .or_else(|| {
                    error!("Redacts field inside content not available, falling back to the one at the event level");
                    redacts
        })
                .expect("At least one redacts field is set"),
        }
}
