use serde::{Deserialize, Serialize};

use crate::{events::key::verification::VerificationMethod, OwnedDeviceId, OwnedUserId};

/// The payload for a key verification request message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.key.verification.request")]
pub struct KeyVerificationRequestEventContent {
    /// A fallback message to alert users that their client does not support the key verification
    /// framework.
    pub body: String,

    /// The verification methods supported by the sender.
    pub methods: Vec<VerificationMethod>,

    /// The device ID which is initiating the request.
    pub from_device: OwnedDeviceId,

    /// The user ID which should receive the request.
    ///
    /// Users should only respond to verification requests if they are named in this field. Users
    /// who are not named in this field and who did not send this event should ignore all other
    /// events that have a `m.reference` relationship with this event.
    pub to: OwnedUserId,
}

impl KeyVerificationRequestEventContent {
    /// Creates a new `KeyVerificationRequestEventContent` with the given body, method, device
    /// and user ID.
    pub fn new(
        body: String,
        methods: Vec<VerificationMethod>,
        from_device: OwnedDeviceId,
        to: OwnedUserId,
    ) -> Self {
        Self { body, methods, from_device, to }
    }
}
