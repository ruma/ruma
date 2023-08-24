//! Types for the [`m.secret.send`] event.
//!
//! [`m.secret.send`]: https://spec.matrix.org/latest/client-server-api/#msecretsend

use std::fmt;

use ruma_common::OwnedTransactionId;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of an `m.secret.send` event.
///
/// An event sent by a client to share a secret with another device, in response to an
/// `m.secret.request` event.
///
/// It must be encrypted as an `m.room.encrypted` event, then sent as a to-device event.
#[derive(Clone, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.secret.send", kind = ToDevice)]
pub struct ToDeviceSecretSendEventContent {
    /// The ID of the request that this is a response to.
    pub request_id: OwnedTransactionId,

    /// The contents of the secret.
    pub secret: String,
}

impl ToDeviceSecretSendEventContent {
    /// Creates a new `SecretSendEventContent` with the given request ID and secret.
    pub fn new(request_id: OwnedTransactionId, secret: String) -> Self {
        Self { request_id, secret }
    }
}

impl fmt::Debug for ToDeviceSecretSendEventContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ToDeviceSecretSendEventContent")
            .field("request_id", &self.request_id)
            .finish_non_exhaustive()
    }
}
