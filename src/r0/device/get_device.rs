//! [GET /_matrix/client/r0/devices/{deviceId}](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-devices-deviceid)

use super::Device;
use ruma_api::ruma_api;
use ruma_identifiers::DeviceId;

ruma_api! {
    metadata {
        description: "Get a device for authenticated user.",
        method: GET,
        name: "get_device",
        path: "/_matrix/client/r0/devices/:device_id",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The device to retrieve.
        #[ruma_api(path)]
        pub device_id: DeviceId,
    }

    response {
        /// Information about the device.
        #[ruma_api(body)]
        pub device: Device,
    }

    error: crate::Error
}
