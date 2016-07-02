//! Types for the *m.call.hangup* event.

use RoomEvent;

/// Sent by either party to signal their termination of the call. This can be sent either once the
/// call has has been established or before to abort the call.
pub type HangupEvent = RoomEvent<HangupEventContent, ()>;

/// The payload of a `HangupEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct HangupEventContent {
    /// The ID of the call this event relates to.
    pub call_id: String,
    /// The version of the VoIP specification this messages adheres to.
    pub version: u64,
}
