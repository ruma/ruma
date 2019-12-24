//! [GET /_matrix/client/r0/devices/{deviceId}](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-devices-deviceid)

use ruma_api::ruma_api;
use ruma_identifiers::DeviceId;
use super::Device;

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
        #[ruma_api(path)]
        device_id: DeviceId,
    }

    response {
        device: Device,
    }
}
