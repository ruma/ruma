//! Types for the [`m.room.redaction`] event.
//!
//! [`m.room.redaction`]: https://spec.matrix.org/latest/client-server-api/#mroomredaction

use js_int::Int;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use self::{
    v1::{
        OriginalRoomRedactionV1Event, OriginalSyncRoomRedactionV1Event, RoomRedactionV1EventContent,
    },
    v11::{
        OriginalRoomRedactionV11Event, OriginalRoomRedactionV1V11CompatEvent,
        OriginalSyncRoomRedactionV11Event, OriginalSyncRoomRedactionV1V11CompatEvent,
        RoomRedactionV11EventContent,
    },
};
use crate::{
    events::{
        BundledMessageLikeRelations, EventContent, MessageLikeEventContent, MessageLikeEventType,
        RedactedMessageLikeEvent, RedactedMessageLikeEventContent, RedactedSyncMessageLikeEvent,
    },
    serde::CanBeEmpty,
    EventId, MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId, OwnedTransactionId, RoomId,
    RoomVersionId, TransactionId, UserId,
};

mod event_serde;
pub mod v1;
pub mod v11;

/// A possibly-redacted redaction event.
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Debug)]
pub enum RoomRedactionEvent {
    /// Original, unredacted form of the event.
    Original(OriginalRoomRedactionEvent),

    /// Redacted form of the event with minimal fields.
    Redacted(RedactedRoomRedactionEvent),
}

impl RoomRedactionEvent {
    /// Returns the `type` of this event.
    pub fn event_type(&self) -> MessageLikeEventType {
        match self {
            Self::Original(ev) => ev.event_type(),
            Self::Redacted(ev) => ev.content.event_type(),
        }
    }

    /// Returns this event's `event_id` field.
    pub fn event_id(&self) -> &EventId {
        match self {
            Self::Original(ev) => ev.event_id(),
            Self::Redacted(ev) => &ev.event_id,
        }
    }

    /// Returns this event's `sender` field.
    pub fn sender(&self) -> &UserId {
        match self {
            Self::Original(ev) => ev.sender(),
            Self::Redacted(ev) => &ev.sender,
        }
    }

    /// Returns this event's `origin_server_ts` field.
    pub fn origin_server_ts(&self) -> MilliSecondsSinceUnixEpoch {
        match self {
            Self::Original(ev) => ev.origin_server_ts(),
            Self::Redacted(ev) => ev.origin_server_ts,
        }
    }

    /// Returns this event's `room_id` field.
    pub fn room_id(&self) -> &RoomId {
        match self {
            Self::Original(ev) => ev.room_id(),
            Self::Redacted(ev) => &ev.room_id,
        }
    }

    /// Get the `redacts` field of this event for the given room version, if available.
    ///
    /// Returns `None` either if the `redacts` field is missing, if it is not in the right place
    /// for the given room version, or if the room version is unknown.
    pub fn redacts(&self, room_version: RoomVersionId) -> Option<&EventId> {
        match self {
            Self::Original(ev) => ev.redacts(room_version),
            Self::Redacted(ev) => ev.content.redacts.as_deref(),
        }
    }

    /// Get the inner `RoomRedactionEvent` if this is an unredacted event.
    pub fn as_original(&self) -> Option<&OriginalRoomRedactionEvent> {
        match self {
            Self::Original(v) => Some(v),
            _ => None,
        }
    }

    /// Get the inner content if this is an unredacted event.
    pub fn original_content(&self) -> Option<RoomRedactionEventContent> {
        self.as_original().map(|ev| ev.content())
    }

    /// Get the `TransactionId` from the unsigned data of this event.
    pub fn transaction_id(&self) -> Option<&TransactionId> {
        match self {
            Self::Original(ev) => ev.transaction_id(),
            _ => None,
        }
    }

    /// Get the `BundledMessageLikeRelations` from the unsigned data of this event.
    pub fn relations(&self) -> Option<BundledMessageLikeRelations<OriginalSyncRoomRedactionEvent>> {
        match self {
            Self::Original(ev) => Some(ev.relations()),
            _ => None,
        }
    }
}

/// Redaction event.
#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum OriginalRoomRedactionEvent {
    /// Redaction event compatible with v1 and v11.
    V1V11Compat(OriginalRoomRedactionV1V11CompatEvent),

    /// Redaction event as defined in room versions 1 through 10.
    V1(OriginalRoomRedactionV1Event),

    /// Redaction event as introduced in room version 11.
    V11(OriginalRoomRedactionV11Event),
}

impl OriginalRoomRedactionEvent {
    /// Returns the `type` of this event.
    pub fn event_type(&self) -> MessageLikeEventType {
        match self {
            Self::V1V11Compat(ev) => ev.content.event_type(),
            Self::V1(ev) => ev.content.event_type(),
            Self::V11(ev) => ev.content.event_type(),
        }
    }

    /// Returns this event's `event_id` field.
    pub fn event_id(&self) -> &EventId {
        match self {
            Self::V1V11Compat(ev) => &ev.event_id,
            Self::V1(ev) => &ev.event_id,
            Self::V11(ev) => &ev.event_id,
        }
    }

    /// Returns this event's `sender` field.
    pub fn sender(&self) -> &UserId {
        match self {
            Self::V1V11Compat(ev) => &ev.sender,
            Self::V1(ev) => &ev.sender,
            Self::V11(ev) => &ev.sender,
        }
    }

    /// Returns this event's `origin_server_ts` field.
    pub fn origin_server_ts(&self) -> MilliSecondsSinceUnixEpoch {
        match self {
            Self::V1V11Compat(ev) => ev.origin_server_ts,
            Self::V1(ev) => ev.origin_server_ts,
            Self::V11(ev) => ev.origin_server_ts,
        }
    }

    /// Returns this event's `room_id` field.
    pub fn room_id(&self) -> &RoomId {
        match self {
            Self::V1V11Compat(ev) => &ev.room_id,
            Self::V1(ev) => &ev.room_id,
            Self::V11(ev) => &ev.room_id,
        }
    }

    /// Get the inner content of this event.
    pub fn content(&self) -> RoomRedactionEventContent {
        match self {
            Self::V1V11Compat(ev) => RoomRedactionEventContent::V11(ev.content.clone()),
            Self::V1(ev) => RoomRedactionEventContent::V1(ev.content.clone()),
            Self::V11(ev) => RoomRedactionEventContent::V11(ev.content.clone()),
        }
    }

    /// Get the `redacts` field of this event for the given room version, if available.
    ///
    /// Returns `None` if the `redacts` field is not in the right place for the given room
    /// version, or if the room version is unknown.
    pub fn redacts(&self, room_version: RoomVersionId) -> Option<&EventId> {
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
            | RoomVersionId::V10 => match self {
                Self::V1V11Compat(ev) => Some(&ev.redacts),
                Self::V1(ev) => Some(&ev.redacts),
                _ => None,
            },
            _ => match self {
                Self::V1V11Compat(ev) => Some(&ev.content.redacts),
                Self::V11(ev) => Some(&ev.content.redacts),
                _ => None,
            },
        }
    }

    /// Get the `TransactionId` from the unsigned data of this event.
    pub fn transaction_id(&self) -> Option<&TransactionId> {
        match self {
            Self::V1V11Compat(ev) => ev.unsigned.transaction_id.as_deref(),
            Self::V1(ev) => ev.unsigned.transaction_id.as_deref(),
            Self::V11(ev) => ev.unsigned.transaction_id.as_deref(),
        }
    }

    /// Get the `BundledMessageLikeRelations` from the unsigned data of this event.
    pub fn relations(&self) -> BundledMessageLikeRelations<OriginalSyncRoomRedactionEvent> {
        match self {
            Self::V1V11Compat(ev) => ev.unsigned.relations.clone().map_replace(|r| r.into()),
            Self::V1(ev) => ev.unsigned.relations.clone().map_replace(|r| r.into()),
            Self::V11(ev) => ev.unsigned.relations.clone().map_replace(|r| r.into()),
        }
    }
}

impl From<OriginalRoomRedactionV1V11CompatEvent> for OriginalRoomRedactionEvent {
    fn from(ev: OriginalRoomRedactionV1V11CompatEvent) -> Self {
        Self::V1V11Compat(ev)
    }
}

impl From<OriginalRoomRedactionV1Event> for OriginalRoomRedactionEvent {
    fn from(ev: OriginalRoomRedactionV1Event) -> Self {
        Self::V1(ev)
    }
}

impl From<OriginalRoomRedactionV11Event> for OriginalRoomRedactionEvent {
    fn from(ev: OriginalRoomRedactionV11Event) -> Self {
        Self::V11(ev)
    }
}

/// Redacted redaction event.
pub type RedactedRoomRedactionEvent = RedactedMessageLikeEvent<RedactedRoomRedactionEventContent>;

/// A possibly-redacted redaction event without a `room_id`.
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Debug)]
pub enum SyncRoomRedactionEvent {
    /// Original, unredacted form of the event.
    Original(OriginalSyncRoomRedactionEvent),

    /// Redacted form of the event with minimal fields.
    Redacted(RedactedSyncRoomRedactionEvent),
}

impl SyncRoomRedactionEvent {
    /// Returns the `type` of this event.
    pub fn event_type(&self) -> MessageLikeEventType {
        match self {
            Self::Original(ev) => ev.event_type(),
            Self::Redacted(ev) => ev.content.event_type(),
        }
    }

    /// Returns this event's `event_id` field.
    pub fn event_id(&self) -> &EventId {
        match self {
            Self::Original(ev) => ev.event_id(),
            Self::Redacted(ev) => &ev.event_id,
        }
    }

    /// Returns this event's `sender` field.
    pub fn sender(&self) -> &UserId {
        match self {
            Self::Original(ev) => ev.sender(),
            Self::Redacted(ev) => &ev.sender,
        }
    }

    /// Returns this event's `origin_server_ts` field.
    pub fn origin_server_ts(&self) -> MilliSecondsSinceUnixEpoch {
        match self {
            Self::Original(ev) => ev.origin_server_ts(),
            Self::Redacted(ev) => ev.origin_server_ts,
        }
    }

    /// Get the `redacts` field of this event for the given room version, if available.
    ///
    /// Returns `None` either if the `redacts` field is missing, if it is not in the right place
    /// for the given room version, or if the room version is unknown.
    pub fn redacts(&self, room_version: &RoomVersionId) -> Option<&EventId> {
        match self {
            Self::Original(ev) => ev.redacts(room_version),
            Self::Redacted(ev) => ev.content.redacts.as_deref(),
        }
    }

    /// Get the inner `SyncRoomRedactionEvent` if this is an unredacted event.
    pub fn as_original(&self) -> Option<&OriginalSyncRoomRedactionEvent> {
        match self {
            Self::Original(v) => Some(v),
            _ => None,
        }
    }

    /// Get the inner content if this is an unredacted event.
    pub fn original_content(&self) -> Option<RoomRedactionEventContent> {
        self.as_original().map(|ev| ev.content())
    }

    /// Get the `TransactionId` from the unsigned data of this event.
    pub fn transaction_id(&self) -> Option<&TransactionId> {
        match self {
            Self::Original(ev) => ev.transaction_id(),
            _ => None,
        }
    }

    /// Get the `BundledMessageLikeRelations` from the unsigned data of this event.
    pub fn relations(&self) -> Option<BundledMessageLikeRelations<OriginalSyncRoomRedactionEvent>> {
        match self {
            Self::Original(ev) => Some(ev.relations()),
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

/// Redaction event without a `room_id`.
#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum OriginalSyncRoomRedactionEvent {
    /// Redaction event compatible with v1 and v11.
    V1V11Compat(OriginalSyncRoomRedactionV1V11CompatEvent),

    /// Redaction event as defined in room versions 1 through 10.
    V1(OriginalSyncRoomRedactionV1Event),

    /// Redaction event as introduced in room version 11.
    V11(OriginalSyncRoomRedactionV11Event),
}

impl OriginalSyncRoomRedactionEvent {
    /// Returns the `type` of this event.
    pub fn event_type(&self) -> MessageLikeEventType {
        match self {
            Self::V1V11Compat(ev) => ev.content.event_type(),
            Self::V1(ev) => ev.content.event_type(),
            Self::V11(ev) => ev.content.event_type(),
        }
    }

    /// Returns this event's `event_id` field.
    pub fn event_id(&self) -> &EventId {
        match self {
            Self::V1V11Compat(ev) => &ev.event_id,
            Self::V1(ev) => &ev.event_id,
            Self::V11(ev) => &ev.event_id,
        }
    }

    /// Returns this event's `sender` field.
    pub fn sender(&self) -> &UserId {
        match self {
            Self::V1V11Compat(ev) => &ev.sender,
            Self::V1(ev) => &ev.sender,
            Self::V11(ev) => &ev.sender,
        }
    }

    /// Returns this event's `origin_server_ts` field.
    pub fn origin_server_ts(&self) -> MilliSecondsSinceUnixEpoch {
        match self {
            Self::V1V11Compat(ev) => ev.origin_server_ts,
            Self::V1(ev) => ev.origin_server_ts,
            Self::V11(ev) => ev.origin_server_ts,
        }
    }

    /// Get the inner content of this event.
    pub fn content(&self) -> RoomRedactionEventContent {
        match self {
            Self::V1V11Compat(ev) => RoomRedactionEventContent::V11(ev.content.clone()),
            Self::V1(ev) => RoomRedactionEventContent::V1(ev.content.clone()),
            Self::V11(ev) => RoomRedactionEventContent::V11(ev.content.clone()),
        }
    }

    /// Get the `redacts` field of this event for the given room version, if available.
    ///
    /// Returns `None` if the `redacts` field is not in the right place for the given room
    /// version, or if the room version is unknown.
    pub fn redacts(&self, room_version: &RoomVersionId) -> Option<&EventId> {
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
            | RoomVersionId::V10 => match self {
                Self::V1V11Compat(ev) => Some(&ev.redacts),
                Self::V1(ev) => Some(&ev.redacts),
                _ => None,
            },
            _ => match self {
                Self::V1V11Compat(ev) => Some(&ev.content.redacts),
                Self::V11(ev) => Some(&ev.content.redacts),
                _ => None,
            },
        }
    }

    /// Get the `TransactionId` from the unsigned data of this event.
    pub fn transaction_id(&self) -> Option<&TransactionId> {
        match self {
            Self::V1V11Compat(ev) => ev.unsigned.transaction_id.as_deref(),
            Self::V1(ev) => ev.unsigned.transaction_id.as_deref(),
            Self::V11(ev) => ev.unsigned.transaction_id.as_deref(),
        }
    }

    /// Get the `BundledMessageLikeRelations` from the unsigned data of this event.
    pub fn relations(&self) -> BundledMessageLikeRelations<OriginalSyncRoomRedactionEvent> {
        match self {
            Self::V1V11Compat(ev) => ev.unsigned.relations.clone().map_replace(Into::into),
            Self::V1(ev) => ev.unsigned.relations.clone().map_replace(Into::into),
            Self::V11(ev) => ev.unsigned.relations.clone().map_replace(Into::into),
        }
    }

    /// Convert this sync event into a full event (one with a `room_id` field).
    pub fn into_full_event(self, room_id: OwnedRoomId) -> OriginalRoomRedactionEvent {
        match self {
            Self::V1V11Compat(ev) => {
                OriginalRoomRedactionEvent::V1V11Compat(ev.into_full_event(room_id))
            }
            Self::V1(ev) => OriginalRoomRedactionEvent::V1(ev.into_full_event(room_id)),
            Self::V11(ev) => OriginalRoomRedactionEvent::V11(ev.into_full_event(room_id)),
        }
    }

    pub(crate) fn into_maybe_redacted(self) -> SyncRoomRedactionEvent {
        SyncRoomRedactionEvent::Original(self)
    }
}

impl From<OriginalSyncRoomRedactionV1V11CompatEvent> for OriginalSyncRoomRedactionEvent {
    fn from(ev: OriginalSyncRoomRedactionV1V11CompatEvent) -> Self {
        Self::V1V11Compat(ev)
    }
}

impl From<OriginalSyncRoomRedactionV1Event> for OriginalSyncRoomRedactionEvent {
    fn from(ev: OriginalSyncRoomRedactionV1Event) -> Self {
        Self::V1(ev)
    }
}

impl From<OriginalSyncRoomRedactionV11Event> for OriginalSyncRoomRedactionEvent {
    fn from(ev: OriginalSyncRoomRedactionV11Event) -> Self {
        Self::V11(ev)
    }
}

impl From<OriginalRoomRedactionEvent> for OriginalSyncRoomRedactionEvent {
    fn from(full: OriginalRoomRedactionEvent) -> Self {
        match full {
            OriginalRoomRedactionEvent::V1V11Compat(ev) => Self::V1V11Compat(ev.into()),
            OriginalRoomRedactionEvent::V1(ev) => Self::V1(ev.into()),
            OriginalRoomRedactionEvent::V11(ev) => Self::V11(ev.into()),
        }
    }
}

/// Redacted redaction event without a `room_id`.
pub type RedactedSyncRoomRedactionEvent =
    RedactedSyncMessageLikeEvent<RedactedRoomRedactionEventContent>;

/// Redaction event content.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(untagged)]
pub enum RoomRedactionEventContent {
    /// Redaction event content as introduced in room version 11.
    V11(RoomRedactionV11EventContent),

    /// Redaction event content as defined in room versions 1 through 10.
    V1(RoomRedactionV1EventContent),
}

impl EventContent for RoomRedactionEventContent {
    type EventType = MessageLikeEventType;

    fn event_type(&self) -> Self::EventType {
        MessageLikeEventType::RoomRedaction
    }
}

impl MessageLikeEventContent for RoomRedactionEventContent {}

/// Redacted redaction event content.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RedactedRoomRedactionEventContent {
    /// The ID of the event that was redacted.
    ///
    /// This is redacted in room versions 10 and below.
    pub redacts: Option<OwnedEventId>,
}

impl EventContent for RedactedRoomRedactionEventContent {
    type EventType = MessageLikeEventType;

    fn event_type(&self) -> MessageLikeEventType {
        MessageLikeEventType::RoomRedaction
    }
}

impl RedactedMessageLikeEventContent for RedactedRoomRedactionEventContent {}

/// Extra information about a redaction that is not incorporated into the event's hash.
#[derive(Clone, Debug, Deserialize)]
#[serde(bound = "E: DeserializeOwned")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RoomRedactionUnsigned<E> {
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
    pub relations: BundledMessageLikeRelations<E>,
}

impl<E> RoomRedactionUnsigned<E> {
    /// Create a new `Unsigned` with fields set to `None`.
    pub fn new() -> Self {
        Self { age: None, transaction_id: None, relations: BundledMessageLikeRelations::default() }
    }
}

impl<E> Default for RoomRedactionUnsigned<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E> CanBeEmpty for RoomRedactionUnsigned<E> {
    /// Whether this unsigned data is empty (all fields are `None`).
    ///
    /// This method is used to determine whether to skip serializing the `unsigned` field in room
    /// events. Do not use it to determine whether an incoming `unsigned` field was present - it
    /// could still have been present but contained none of the known fields.
    fn is_empty(&self) -> bool {
        self.age.is_none() && self.transaction_id.is_none() && self.relations.is_empty()
    }
}

/// Content of a non-redacted room redaction event.
pub trait RedactionEventContent: MessageLikeEventContent {}

/// Content of a redacted room redaction event.
pub trait RedactedRedactionEventContent: RedactedMessageLikeEventContent {}
