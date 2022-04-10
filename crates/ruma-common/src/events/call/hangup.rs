//! Types for the [`m.call.hangup`] event.
//!
//! [`m.call.hangup`]: https://spec.matrix.org/v1.2/client-server-api/#mcallhangup

use js_int::UInt;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{serde::StringEnum, PrivOwnedStr};

/// The content of an `m.call.hangup` event.
///
/// Sent by either party to signal their termination of the call. This can be sent either once the
/// call has has been established or before to abort the call.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.call.hangup", kind = MessageLike)]
pub struct CallHangupEventContent {
    /// The ID of the call this event relates to.
    pub call_id: String,

    /// The version of the VoIP specification this messages adheres to.
    pub version: UInt,

    /// Optional error reason for the hangup.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<Reason>,
}

impl CallHangupEventContent {
    /// Creates a new `HangupEventContent` with the given call ID and VoIP version.
    pub fn new(call_id: String, version: UInt) -> Self {
        Self { call_id, version, reason: None }
    }
}

/// A reason for a hangup.
///
/// Should not be provided when the user naturally ends or rejects the call. When there was an error
/// in the call negotiation, this should be `ice_failed` for when ICE negotiation fails or
/// `invite_timeout` for when the other party did not answer in time.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Reason {
    /// ICE negotiation failure.
    IceFailed,

    /// Party did not answer in time.
    InviteTimeout,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl Reason {
    /// Creates a string slice from this `Reason`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}
