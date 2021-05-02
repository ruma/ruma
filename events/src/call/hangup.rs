//! Types for the *m.call.hangup* event.

use js_int::UInt;
use ruma_events_macros::MessageEventContent;
use ruma_serde::StringEnum;
use serde::{Deserialize, Serialize};

use crate::MessageEvent;

/// Sent by either party to signal their termination of the call. This can be sent either once the
/// call has has been established or before to abort the call.
pub type HangupEvent = MessageEvent<HangupEventContent>;

/// The payload for `HangupEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, MessageEventContent)]
#[ruma_event(type = "m.call.hangup")]
pub struct HangupEventContent {
    /// The ID of the call this event relates to.
    pub call_id: String,

    /// The version of the VoIP specification this messages adheres to.
    pub version: UInt,

    /// Optional error reason for the hangup.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<Reason>,
}

/// A reason for a hangup.
///
/// This should not be provided when the user naturally ends or rejects the call. When there was an
/// error in the call negotiation, this should be `ice_failed` for when ICE negotiation fails or
/// `invite_timeout` for when the other party did not answer in time.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
pub enum Reason {
    /// ICE negotiation failure.
    IceFailed,

    /// Party did not answer in time.
    InviteTimeout,

    #[doc(hidden)]
    _Custom(String),
}
