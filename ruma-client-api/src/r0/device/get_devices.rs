//! [GET /_matrix/client/r0/devices](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-devices)

use super::Device;
use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Get registered devices for authenticated user.",
        method: GET,
        name: "get_devices",
        path: "/_matrix/client/r0/devices",
        rate_limited: false,
        authentication: AccessToken,
    }

    #[derive(Default)]
    request: {}

    response: {
        /// A list of all registered devices for this user
        pub devices: Vec<Device>,
    }

    error: crate::Error
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self
    }
}

impl Response {
    /// Creates a new `Response` with the given devices.
    pub fn new(devices: Vec<Device>) -> Self {
        Self { devices }
    }
}
