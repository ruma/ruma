//! Types for the [`m.call.hangup`] event.
//!
//! [`m.call.hangup`]: https://spec.matrix.org/latest/client-server-api/#mcallhangup

use ruma_common::{serde::StringEnum, OwnedVoipId, VoipVersionId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::PrivOwnedStr;

/// The content of an `m.call.hangup` event.
///
/// Sent by either party to signal their termination of the call.
///
/// In VoIP version 0, this can be sent either once the call has been established or before to abort
/// the call.
///
/// If the call is using VoIP version 1, this should only be sent by the caller after sending the
/// invite or by the callee after answering the invite. To reject an invite, send an
/// [`m.call.reject`] event.
///
/// [`m.call.reject`]: super::reject::CallRejectEventContent
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.call.hangup", kind = MessageLike)]
pub struct CallHangupEventContent {
    /// A unique identifier for the call.
    pub call_id: OwnedVoipId,

    /// **Required in VoIP version 1.** A unique ID for this session for the duration of the call.
    ///
    /// Must be the same as the one sent by the previous invite or answer from
    /// this session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub party_id: Option<OwnedVoipId>,

    /// The version of the VoIP specification this messages adheres to.
    pub version: VoipVersionId,

    /// Error reason for the hangup.
    ///
    /// Defaults to `Reason::UserHangup`.
    #[serde(default)]
    pub reason: Reason,
}

impl CallHangupEventContent {
    /// Creates a new `CallHangupEventContent` with the given call ID and VoIP version.
    pub fn new(call_id: OwnedVoipId, version: VoipVersionId) -> Self {
        Self { call_id, party_id: None, version, reason: Default::default() }
    }

    /// Convenience method to create a VoIP version 0 `CallHangupEventContent` with all the required
    /// fields.
    pub fn version_0(call_id: OwnedVoipId) -> Self {
        Self::new(call_id, VoipVersionId::V0)
    }

    /// Convenience method to create a VoIP version 1 `CallHangupEventContent` with all the required
    /// fields.
    pub fn version_1(call_id: OwnedVoipId, party_id: OwnedVoipId, reason: Reason) -> Self {
        Self { call_id, party_id: Some(party_id), version: VoipVersionId::V1, reason }
    }
}

/// A reason for a hangup.
///
/// Should not be provided when the user naturally ends or rejects the call. When there was an error
/// in the call negotiation, this should be `ice_failed` for when ICE negotiation fails or
/// `invite_timeout` for when the other party did not answer in time.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Default, PartialEq, Eq, StringEnum)]
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
    IceTimeout,

    /// The user chose to end the call.
    #[default]
    UserHangup,

    /// The client was unable to start capturing media in such a way as it is unable to continue
    /// the call.
    UserMediaFailed,

    /// The user is busy.
    UserBusy,

    /// Some other failure occurred that meant the client was unable to continue the call rather
    /// than the user choosing to end it.
    UnknownError,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
