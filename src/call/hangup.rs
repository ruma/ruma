//! Types for the *m.call.hangup* event.

use core::EventType;

/// Sent by either party to signal their termination of the call. This can be sent either once the
/// call has has been established or before to abort the call.
pub struct HangupEvent {
    pub content: HangupEventContent,
    pub event_id: String,
    pub event_type: EventType,
    pub room_id: String,
    pub user_id: String,
}

/// The payload of a `HangupEvent`.
pub struct HangupEventContent {
    /// The ID of the call this event relates to.
    pub call_id: String,
    /// The version of the VoIP specification this messages adheres to.
    pub version: u64,
}
