//! Types for the `m.room.redaction` event as introduced in room version 11.

use ruma_macros::{Event, EventContent};
use serde::{Deserialize, Serialize};

use crate::{
    events::RedactContent, MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId, OwnedUserId,
    RoomVersionId,
};

use super::{
    v1::{OriginalRedactionV1Event, OriginalSyncRedactionV1Event},
    RedactedRoomRedactionEventContent, RedactionEventContent, RoomRedactionUnsigned,
};

/// Redaction event as introduced in room version 11.
#[derive(Clone, Debug, Event)]
#[allow(clippy::exhaustive_structs)]
pub struct OriginalRoomRedactionV11Event {
    /// Data specific to the event type.
    pub content: RoomRedactionV11EventContent,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: OwnedEventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// The ID of the room associated with this event.
    pub room_id: OwnedRoomId,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: RoomRedactionUnsigned<OriginalSyncRoomRedactionV11Event>,
}

/// Redaction event compatible with v1 and v11.
pub type OriginalRoomRedactionV1V11CompatEvent =
    OriginalRedactionV1Event<RoomRedactionV11EventContent>;

/// Redaction event without a `room_id`, as introduced in room version 11.
#[derive(Clone, Debug, Event)]
#[allow(clippy::exhaustive_structs)]
pub struct OriginalSyncRoomRedactionV11Event {
    /// Data specific to the event type.
    pub content: RoomRedactionV11EventContent,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: OwnedEventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: RoomRedactionUnsigned<OriginalSyncRoomRedactionV11Event>,
}

/// Redaction event compatible with v1 and v11.
pub type OriginalSyncRoomRedactionV1V11CompatEvent =
    OriginalSyncRedactionV1Event<RoomRedactionV11EventContent>;

/// A redaction of an event as introduced in room version 11.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.redaction", kind = MessageLike, custom_redacted)]
pub struct RoomRedactionV11EventContent {
    /// The ID of the event that was redacted.
    pub redacts: OwnedEventId,

    /// The reason for the redaction, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl RoomRedactionV11EventContent {
    /// Creates a new `RoomRedactionV1EventContent` which redacts the event with the given ID.
    pub fn new(redacts: OwnedEventId) -> Self {
        Self { redacts, reason: None }
    }

    /// Creates a new `RoomRedactionV1EventContent` which redacts the event with the given ID for
    /// the given reason.
    pub fn with_reason(redacts: OwnedEventId, reason: String) -> Self {
        Self { redacts, reason: Some(reason) }
    }
}

impl RedactionEventContent for RoomRedactionV11EventContent {}

impl RedactContent for RoomRedactionV11EventContent {
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
            _ => Some(self.redacts),
        };

        RedactedRoomRedactionEventContent { redacts }
    }
}
