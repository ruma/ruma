//! Types for the *m.key.verification.request* event.

use std::time::SystemTime;

use ruma_events_macros::BasicEventContent;
use ruma_identifiers::DeviceId;
use serde::{Deserialize, Serialize};

use super::VerificationMethod;
use crate::BasicEvent;

/// Requests a key verification with another user's devices.
///
/// Typically sent as a to-device event.
pub type RequestEvent = BasicEvent<RequestEventContent>;

/// The payload for `RequestEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, BasicEventContent)]
#[ruma_event(type = "m.key.verification.request")]
#[ruma_event(not_redacted)]
pub struct RequestEventContent {
    /// The device ID which is initiating the request.
    pub from_device: DeviceId,

    /// An opaque identifier for the verification request.
    ///
    /// Must be unique with respect to the devices involved.
    pub transaction_id: String,

    /// The verification methods supported by the sender.
    pub methods: Vec<VerificationMethod>,

    /// The time in milliseconds for when the request was made.
    ///
    /// If the request is in the future by more than 5 minutes or more than 10 minutes in
    /// the past, the message should be ignored by the receiver.
    #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
    pub timestamp: SystemTime,
}
