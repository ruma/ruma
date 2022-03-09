//! Types for the [`m.key.verification.request`] event.
//!
//! [`m.key.verification.request`]: https://spec.matrix.org/v1.2/client-server-api/#mkeyverificationrequest

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::VerificationMethod;
use crate::{DeviceId, MilliSecondsSinceUnixEpoch, TransactionId};

/// The content of an `m.key.verification.request` event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.key.verification.request", kind = ToDevice)]
pub struct ToDeviceKeyVerificationRequestEventContent {
    /// The device ID which is initiating the request.
    pub from_device: Box<DeviceId>,

    /// An opaque identifier for the verification request.
    ///
    /// Must be unique with respect to the devices involved.
    pub transaction_id: Box<TransactionId>,

    /// The verification methods supported by the sender.
    pub methods: Vec<VerificationMethod>,

    /// The time in milliseconds for when the request was made.
    ///
    /// If the request is in the future by more than 5 minutes or more than 10 minutes in
    /// the past, the message should be ignored by the receiver.
    pub timestamp: MilliSecondsSinceUnixEpoch,
}

impl ToDeviceKeyVerificationRequestEventContent {
    /// Creates a new `ToDeviceKeyVerificationRequestEventContent` with the given device ID,
    /// transaction ID, methods and timestamp.
    pub fn new(
        from_device: Box<DeviceId>,
        transaction_id: Box<TransactionId>,
        methods: Vec<VerificationMethod>,
        timestamp: MilliSecondsSinceUnixEpoch,
    ) -> Self {
        Self { from_device, transaction_id, methods, timestamp }
    }
}
