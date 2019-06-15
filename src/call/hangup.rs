//! Types for the *m.call.hangup* event.

use serde::{Deserialize, Serialize};

room_event! {
    /// Sent by either party to signal their termination of the call. This can be sent either once
    /// the call has has been established or before to abort the call.
    pub struct HangupEvent(HangupEventContent) {}
}

/// The payload of a `HangupEvent`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct HangupEventContent {
    /// The ID of the call this event relates to.
    pub call_id: String,

    /// The version of the VoIP specification this messages adheres to.
    pub version: u64,

    /// Optional error reason for the hangup.
    pub reason: Option<Reason>,
}

/// A reason for a hangup.
///
/// This should not be provided when the user naturally ends or rejects the call. When there was an
/// error in the call negotiation, this should be `ice_failed` for when ICE negotiation fails or
/// `invite_timeout` for when the other party did not answer in time.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum Reason {
    /// ICE negotiation failure.
    #[serde(rename = "ice_failed")]
    IceFailed,

    /// Party did not answer in time.
    #[serde(rename = "invite_timeout")]
    InviteTimeout,

    /// Additional variants may be added in the future and will not be considered breaking changes
    /// to `ruma-events`.
    #[doc(hidden)]
    #[serde(skip)]
    __Nonexhaustive,
}

impl_enum! {
    Reason {
        IceFailed => "ice_failed",
        InviteTimeout => "invite_timeout",
    }
}
