//! [DELETE /_matrix/client/r0/devices/{deviceId}](https://matrix.org/docs/spec/client_server/r0.6.0#delete-matrix-client-r0-devices-deviceid)

use ruma_api::ruma_api;
use ruma_identifiers::DeviceId;

use crate::r0::uiaa::{AuthData, IncomingAuthData, UiaaResponse};

ruma_api! {
    metadata: {
        description: "Delete a device for authenticated user.",
        method: DELETE,
        name: "delete_device",
        path: "/_matrix/client/r0/devices/:device_id",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The device to delete.
        #[ruma_api(path)]
        pub device_id: &'a DeviceId,

        /// Additional authentication information for the user-interactive authentication API.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub auth: Option<AuthData<'a>>,
    }

    #[derive(Default)]
    response: {}

    error: UiaaResponse
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given device ID.
    pub fn new(device_id: &'a DeviceId) -> Self {
        Self { device_id, auth: None }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
