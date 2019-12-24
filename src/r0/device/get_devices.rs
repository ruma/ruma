//! [GET /_matrix/client/r0/devices](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-devices)

use ruma_api::ruma_api;
use super::Device;

ruma_api! {
    metadata {
        description: "Get registered devices for authenticated user.",
        method: GET,
        name: "get_devices",
        path: "/_matrix/client/r0/devices",
        rate_limited: false,
        requires_authentication: true,
    }

    request {}

    response {
        devices: Vec<Device>
    }
}
