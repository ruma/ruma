use ruma_common::{OwnedDeviceId, OwnedUserId};
use serde::{Deserialize, Serialize};

use super::FormattedBody;
use crate::key::verification::VerificationMethod;

/// The payload for a key verification request message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.key.verification.request")]
pub struct KeyVerificationRequestEventContent {
    /// A fallback message to alert users that their client does not support the key verification
    /// framework.
    ///
    /// Clients that do support the key verification framework should hide the body and instead
    /// present the user with an interface to accept or reject the key verification.
    pub body: String,

    /// Formatted form of the `body`.
    ///
    /// As with the `body`, clients that do support the key verification framework should hide the
    /// formatted body and instead present the user with an interface to accept or reject the key
    /// verification.
    #[serde(flatten)]
    pub formatted: Option<FormattedBody>,

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
        Self { body, formatted: None, methods, from_device, to }
    }
}
