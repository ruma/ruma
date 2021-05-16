//! Types for the *m.key.verification.request* event.

use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_events_macros::EventContent;
use ruma_identifiers::DeviceIdBox;
use serde::{Deserialize, Serialize};

use super::VerificationMethod;

/// The payload for `RequestEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.key.verification.request", kind = ToDevice)]
pub struct RequestToDeviceEventContent {
    /// The device ID which is initiating the request.
    pub from_device: DeviceIdBox,

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
    pub timestamp: MilliSecondsSinceUnixEpoch,
}

impl RequestToDeviceEventContent {
    /// Creates a new `RequestToDeviceEventContent` with the given device ID, transaction ID,
    /// methods and timestamp.
    pub fn new(
        from_device: DeviceIdBox,
        transaction_id: String,
        methods: Vec<VerificationMethod>,
        timestamp: MilliSecondsSinceUnixEpoch,
    ) -> Self {
        Self { from_device, transaction_id, methods, timestamp }
    }
}
