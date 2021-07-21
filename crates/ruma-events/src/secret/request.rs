//! Types for the *m.secret.request* event.

use ruma_events_macros::EventContent;
use ruma_identifiers::DeviceIdBox;
use ruma_serde::StringEnum;
use serde::{Deserialize, Serialize};

use crate::ToDeviceEvent;

/// Event sent by a client to request a secret from another device or to cancel a previous request.
///
/// It is sent as an unencrypted to-device event.
pub type SecretRequestEvent = ToDeviceEvent<SecretRequestEventContent>;

/// The payload for SecretRequestEvent
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.secret.request", kind = ToDevice)]
pub struct SecretRequestEventContent {
    /// The name of the secret that is being requested.
    ///
    /// Required if `action` is `request`.
    pub name: Option<String>,

    /// One of ["request", "request_cancellation"].
    pub action: RequestAction,

    /// The ID of the device requesting the event.
    pub requesting_device_id: DeviceIdBox,

    /// A random string uniquely identifying (with respect to the requester and the target) the
    /// target for a secret.
    ///
    /// If the secret is requested from multiple devices at the same time, the same ID may be used
    /// for every target. The same ID is also used in order to cancel a previous request.
    pub request_id: String,
}

impl SecretRequestEventContent {
    /// Creates a new `SecretRequestEventContent` with the given name, action, requesting device ID
    /// and request ID.
    pub fn new(
        name: Option<String>,
        action: RequestAction,
        requesting_device_id: DeviceIdBox,
        request_id: String,
    ) -> Self {
        Self { name, action, requesting_device_id, request_id }
    }
}

/// Action for a *m.secret.request* event.
#[derive(Clone, Debug, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum RequestAction {
    /// Request a secret.
    Request,

    /// Cancel a request for a secret.
    RequestCancellation,

    #[doc(hidden)]
    _Custom(String),
}
