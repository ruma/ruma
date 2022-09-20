//! Types for the [`m.call.hangup`] event.
//!
//! [`m.call.hangup`]: https://spec.matrix.org/v1.2/client-server-api/#mcallhangup

use ruma_macros::EventContent;
#[cfg(feature = "unstable-msc2746")]
use serde::Serializer;
use serde::{Deserialize, Serialize};

use crate::{serde::StringEnum, OwnedVoipId, PrivOwnedStr, VoipVersionId};

/// The content of an `m.call.hangup` event.
///
/// Sent by either party to signal their termination of the call.
///
/// In VoIP version 0, this can be sent either once the call has been established or before to abort
/// the call.
///
/// With the `unstable-msc2746` feature, and if the call is using VoIP version 1, this should only
/// be sent by the caller after sending the invite or by the callee after answering the invite. To
/// reject an invite, send an [`m.call.reject`] event.
///
/// [`m.call.reject`]: super::reject::CallRejectEventContent
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.call.hangup", kind = MessageLike)]
pub struct CallHangupEventContent {
    /// A unique identifier for the call.
    pub call_id: OwnedVoipId,

    #[cfg(feature = "unstable-msc2746")]
    /// **Required in VoIP version 1.** A unique ID for this session for the duration of the call.
    ///
    /// Must be the same as the one sent by the previous invite or answer from
    /// this session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub party_id: Option<OwnedVoipId>,

    /// The version of the VoIP specification this messages adheres to.
    pub version: VoipVersionId,

    /// Optional error reason for the hangup.
    ///
    /// With the `unstable-msc2746` feature, this field defaults to `Some(Reason::UserHangup)`.
    #[cfg_attr(not(feature = "unstable-msc2746"), serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        feature = "unstable-msc2746",
        serde(
            default = "Reason::option_with_default",
            serialize_with = "Reason::serialize_option_with_default"
        )
    )]
    pub reason: Option<Reason>,
}

impl CallHangupEventContent {
    /// Creates a new `CallHangupEventContent` with the given call ID and VoIP version.
    pub fn new(call_id: OwnedVoipId, version: VoipVersionId) -> Self {
        Self {
            call_id,
            #[cfg(feature = "unstable-msc2746")]
            party_id: None,
            version,
            reason: Default::default(),
        }
    }

    /// Convenience method to create a VoIP version 0 `CallHangupEventContent` with all the required
    /// fields.
    pub fn version_0(call_id: OwnedVoipId) -> Self {
        Self::new(call_id, VoipVersionId::V0)
    }

    /// Convenience method to create a VoIP version 1 `CallHangupEventContent` with all the required
    /// fields.
    #[cfg(feature = "unstable-msc2746")]
    pub fn version_1(call_id: OwnedVoipId, party_id: OwnedVoipId, reason: Reason) -> Self {
        Self { call_id, party_id: Some(party_id), version: VoipVersionId::V1, reason: Some(reason) }
    }
}

/// A reason for a hangup.
///
/// Should not be provided when the user naturally ends or rejects the call. When there was an error
/// in the call negotiation, this should be `ice_failed` for when ICE negotiation fails or
/// `invite_timeout` for when the other party did not answer in time.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[cfg_attr(feature = "unstable-msc2746", derive(Default))]
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
    #[default]
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
    #[cfg(feature = "unstable-msc2746")]
    fn serialize_option_with_default<S>(
        reason: &Option<Reason>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(reason) = &reason {
            reason.serialize(serializer)
        } else {
            Self::default().serialize(serializer)
        }
    }

    #[cfg(feature = "unstable-msc2746")]
    fn option_with_default() -> Option<Self> {
        Some(Self::default())
    }
}
