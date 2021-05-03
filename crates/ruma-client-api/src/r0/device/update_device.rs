//! [PUT /_matrix/client/r0/devices/{deviceId}](https://matrix.org/docs/spec/client_server/r0.6.0#put-matrix-client-r0-devices-deviceid)

use ruma_api::ruma_api;
use ruma_identifiers::DeviceId;

ruma_api! {
    metadata: {
        description: "Update metadata for a device.",
        method: PUT,
        name: "update_device",
        path: "/_matrix/client/r0/devices/:device_id",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The device to update.
        #[ruma_api(path)]
        pub device_id: &'a DeviceId,

        /// The new display name for this device. If this is `None`, the display name won't be
        /// changed.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub display_name: Option<String>,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given device ID.
    pub fn new(device_id: &'a DeviceId) -> Self {
        Self { device_id, display_name: None }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
