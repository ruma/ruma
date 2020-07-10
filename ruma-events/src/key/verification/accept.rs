//! Types for the *m.key.verification.accept* event.

use ruma_events_macros::BasicEventContent;
use serde::{Deserialize, Serialize};

use super::{
    HashAlgorithm, KeyAgreementProtocol, MessageAuthenticationCode, ShortAuthenticationString,
    VerificationMethod,
};
use crate::BasicEvent;

/// Accepts a previously sent *m.key.verification.start* message.
///
/// Typically sent as a to-device event.
pub type AcceptEvent = BasicEvent<AcceptEventContent>;

/// The payload for `AcceptEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, BasicEventContent)]
#[ruma_event(type = "m.key.verification.accept")]
#[ruma_event(not_redacted)]
pub struct AcceptEventContent {
    /// An opaque identifier for the verification process.
    ///
    /// Must be the same as the one used for the *m.key.verification.start* message.
    pub transaction_id: String,

    /// The verification method to use.
    ///
    /// Must be `m.sas.v1`.
    pub method: VerificationMethod,

    /// The key agreement protocol the device is choosing to use, out of the options in the
    /// *m.key.verification.start* message.
    pub key_agreement_protocol: KeyAgreementProtocol,

    /// The hash method the device is choosing to use, out of the options in the
    /// *m.key.verification.start* message.
    pub hash: HashAlgorithm,

    /// The message authentication code the device is choosing to use, out of the options in the
    /// *m.key.verification.start* message.
    pub message_authentication_code: MessageAuthenticationCode,

    /// The SAS methods both devices involved in the verification process understand.
    ///
    /// Must be a subset of the options in the *m.key.verification.start* message.
    pub short_authentication_string: Vec<ShortAuthenticationString>,

    /// The hash (encoded as unpadded base64) of the concatenation of the device's ephemeral public
    /// key (encoded as unpadded base64) and the canonical JSON representation of the
    /// *m.key.verification.start* message.
    pub commitment: String,
}
