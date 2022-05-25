//! Types for the [`m.call.hangup`] event.
//!
//! [`m.call.hangup`]: https://spec.matrix.org/v1.2/client-server-api/#mcallhangup

#[cfg(feature = "unstable-msc2746")]
use js_int::uint;
#[cfg(not(feature = "unstable-msc2746"))]
use js_int::UInt;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

#[cfg(feature = "unstable-msc2746")]
use super::CallVersion;
#[cfg(feature = "unstable-msc2746")]
use crate::OwnedVoipId;
use crate::{serde::StringEnum, PrivOwnedStr};

/// The content of an `m.call.hangup` event.
///
/// Sent by either party to signal their termination of the call.
///
/// In version 0, this can be sent either once the call has been established or before to abort
/// the call.
///
/// With the `unstable-msc2746` feature, and if the call is in version 1, this should only be sent
/// by the caller after sending the invite or by the callee after answering the invite. To reject an
/// invite, send an [`m.call.reject`] event.
///
/// [`m.call.reject`]: super::reject::CallRejectEventContent
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.call.hangup", kind = MessageLike)]
pub struct CallHangupEventContent {
    #[cfg(not(feature = "unstable-msc2746"))]
    /// A unique identifier for the call.
    ///
    /// With the `unstable-msc2746` feature, this uses the stricter `VoipId` type.
    pub call_id: String,

    #[cfg(feature = "unstable-msc2746")]
    /// A unique identifier for the call.
    ///
    /// Without the `unstable-msc2746` feature, this can be any string.
    pub call_id: OwnedVoipId,

    #[cfg(feature = "unstable-msc2746")]
    /// **Required in version 1.** A unique ID for this session for the duration of the call.
    ///
    /// Must be the same as the one sent by the previous invite or answer from
    /// this session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub party_id: Option<OwnedVoipId>,

    #[cfg(not(feature = "unstable-msc2746"))]
    /// The version of the VoIP specification this messages adheres to.
    ///
    /// With the `unstable-msc2746` feature, this can be a `UInt` or a `String`.
    pub version: UInt,

    #[cfg(feature = "unstable-msc2746")]
    /// The version of the VoIP specification this messages adheres to.
    ///
    /// Without the `unstable-msc2746` feature, this is a `UInt`.
    pub version: CallVersion,

    /// Optional error reason for the hangup.
    ///
    /// Without the `unstable-msc2746` feature, this field is required.
    #[cfg(not(feature = "unstable-msc2746"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<Reason>,

    /// Error reason for the hangup.
    ///
    /// Defaults to `Reason::UserHangup` if it is missing.
    ///
    /// Without the `unstable-msc2746` feature, this field is optional.
    #[cfg(feature = "unstable-msc2746")]
    #[serde(default)]
    pub reason: Reason,
}

impl CallHangupEventContent {
    /// Creates a new `CallHangupEventContent` with the given call ID and VoIP version.
    ///
    /// With the `unstable-msc2746` feature, this method takes an `OwnedVoipId` for the call ID and
    /// a `CallVersion` for the version.
    #[cfg(not(feature = "unstable-msc2746"))]
    pub fn new(call_id: String, version: UInt) -> Self {
        Self { call_id, version, reason: None }
    }

    /// Creates a new `CallHangupEventContent` with the given call ID and VoIP version.
    ///
    /// Without the `unstable-msc2746` feature, this method takes a `String` for the call ID and a
    /// `UInt` for the version.
    #[cfg(feature = "unstable-msc2746")]
    pub fn new(call_id: OwnedVoipId, version: CallVersion) -> Self {
        Self { call_id, party_id: None, version, reason: Default::default() }
    }

    /// Convenience method to create a version 0 `CallHangupEventContent` with all the required
    /// fields.
    #[cfg(feature = "unstable-msc2746")]
    pub fn version_0(call_id: OwnedVoipId) -> Self {
        Self::new(call_id, uint!(0).into())
    }

    /// Convenience method to create a version 1 `CallHangupEventContent` with all the required
    /// fields.
    #[cfg(feature = "unstable-msc2746")]
    pub fn version_1(call_id: OwnedVoipId, party_id: OwnedVoipId, reason: Reason) -> Self {
        Self { call_id, party_id: Some(party_id), version: uint!(1).into(), reason }
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

    /// The connection failed after some media was exchanged.
    ///
    /// Note that, in the case of an ICE renegotiation, a client should be sure to send
    /// `ice_timeout` rather than `ice_failed` if media had previously been received successfully,
    /// even if the ICE renegotiation itself failed.
    #[cfg(feature = "unstable-msc2746")]
    IceTimeout,

    /// The user chose to end the call.
    #[cfg(feature = "unstable-msc2746")]
    UserHangup,

    /// The client was unable to start capturing media in such a way as it is unable to continue
    /// the call.
    #[cfg(feature = "unstable-msc2746")]
    UserMediaFailed,

    /// The user is busy.
    #[cfg(feature = "unstable-msc2746")]
    UserBusy,

    /// Some other failure occurred that meant the client was unable to continue the call rather
    /// than the user choosing to end it.
    #[cfg(feature = "unstable-msc2746")]
    UnknownError,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl Reason {
    /// Creates a string slice from this `Reason`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

#[cfg(feature = "unstable-msc2746")]
impl Default for Reason {
    fn default() -> Self {
        Self::UserHangup
    }
}
