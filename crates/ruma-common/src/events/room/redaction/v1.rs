//! Types for the `m.room.redaction` event as defined in room versions 1 through 10.

use ruma_macros::{Event, EventContent};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::{
    events::RedactContent, MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId, OwnedUserId,
    RoomVersionId,
};

use super::{RedactedRoomRedactionEventContent, RedactionEventContent, RoomRedactionUnsigned};

/// Redaction event as defined in room versions 1 through 10.
pub type OriginalRoomRedactionV1Event = OriginalRedactionV1Event<RoomRedactionV1EventContent>;

/// Redaction event as defined in room versions 1 through 10.
pub type OriginalSyncRoomRedactionV1Event =
    OriginalSyncRedactionV1Event<RoomRedactionV1EventContent>;

/// Redaction event as defined in room versions 1 through 10.
#[derive(Clone, Debug, Event)]
#[allow(clippy::exhaustive_structs)]
pub struct OriginalRedactionV1Event<C>
where
    C: RedactionEventContent,
{
    /// Data specific to the event type.
    pub content: C,

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
    pub unsigned: RoomRedactionUnsigned<OriginalSyncRedactionV1Event<C>>,
}

/// Redaction event without a `room_id` as defined in room versions 1 through 10.
#[derive(Clone, Debug, Event)]
#[allow(clippy::exhaustive_structs)]
pub struct OriginalSyncRedactionV1Event<C>
where
    C: RedactionEventContent,
{
    /// Data specific to the event type.
    pub content: C,

    /// The ID of the event that was redacted.
    pub redacts: OwnedEventId,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: OwnedEventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: RoomRedactionUnsigned<OriginalSyncRedactionV1Event<C>>,
}

/// A redaction of an event as defined in room versions 1 through 10.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.redaction", kind = MessageLike, custom_redacted)]
pub struct RoomRedactionV1EventContent {
    /// The reason for the redaction, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl RoomRedactionV1EventContent {
    /// Creates an empty `RoomRedactionV1EventContent`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `RoomRedactionV1EventContent` with the given reason.
    pub fn with_reason(reason: String) -> Self {
        Self { reason: Some(reason) }
    }
}

impl RedactionEventContent for RoomRedactionV1EventContent {}

impl RedactContent for RoomRedactionV1EventContent {
    type Redacted = RedactedRoomRedactionEventContent;

    fn redact(self, version: &RoomVersionId) -> Self::Redacted {
        match version {
            RoomVersionId::V1
            | RoomVersionId::V2
            | RoomVersionId::V3
            | RoomVersionId::V4
            | RoomVersionId::V5
            | RoomVersionId::V6
            | RoomVersionId::V7
            | RoomVersionId::V8
            | RoomVersionId::V9
            | RoomVersionId::V10 => {}
            _ => warn!("Trying to apply {version:?} redaction algorithm to pre-v11 RoomRedactionV1EventContent"),
        }

        RedactedRoomRedactionEventContent::default()
    }
}
