//! Types for the *m.secret.send* event.

use ruma_events_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::ToDeviceEvent;

/// An event sent by a client to share a secret with another device, in response to an
/// `m.secret.request` event.
///
/// It must be encrypted as an `m.room.encrypted` event, then sent as a to-device
pub type SecretSendEvent = ToDeviceEvent<SecretSendEventContent>;

/// The payload for `SecretSendEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.secret.send", kind = ToDevice)]
pub struct SecretSendEventContent {
    /// The ID of the request that this is a response to.
    pub request_id: String,

    /// The contents of the secret.
    pub secret: String,
}

impl SecretSendEventContent {
    /// Creates a new `SecretSendEventContent` with the given request ID and secret.
    pub fn new(request_id: String, secret: String) -> Self {
        Self { request_id, secret }
    }
}
