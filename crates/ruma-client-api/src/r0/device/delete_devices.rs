//! [POST /_matrix/client/r0/delete_devices](https://matrix.org/docs/spec/client_server/r0.6.1#post-matrix-client-r0-delete-devices)

use ruma_api::ruma_api;
use ruma_identifiers::DeviceId;

use crate::r0::uiaa::{AuthData, IncomingAuthData, UiaaResponse};

ruma_api! {
    metadata: {
        description: "Delete specified devices.",
        method: POST,
        r0: "/_matrix/client/r0/delete_devices",
        stable: "/_matrix/client/v3/delete_devices",
        name: "delete_devices",
        rate_limited: false,
        authentication: AccessToken,
        added: 1.0,
    }

    request: {
        /// List of devices to delete.
        pub devices: &'a [Box<DeviceId>],

        /// Additional authentication information for the user-interactive authentication API.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub auth: Option<AuthData<'a>>,
    }

    #[derive(Default)]
    response: {}

    error: UiaaResponse
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given device list.
    pub fn new(devices: &'a [Box<DeviceId>]) -> Self {
        Self { devices, auth: None }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self {}
    }
}
