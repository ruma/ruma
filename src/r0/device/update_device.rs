//! [PUT /_matrix/client/r0/devices/{deviceId}](https://matrix.org/docs/spec/client_server/r0.6.0#put-matrix-client-r0-devices-deviceid)

use ruma_api::ruma_api;
use ruma_identifiers::DeviceId;

ruma_api! {
    metadata {
        description: "Update metadata for a device.",
        method: PUT,
        name: "update_device",
        path: "/_matrix/client/r0/devices/:device_id",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        #[ruma_api(path)]
        device_id: DeviceId,
        #[serde(skip_serializing_if = "Option::is_none")]
        display_name: Option<String>,
    }

    response {}
}
