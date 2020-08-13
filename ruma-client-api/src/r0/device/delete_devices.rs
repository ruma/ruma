//! [POST /_matrix/client/r0/delete_devices](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-delete-devices)

use ruma_api::ruma_api;
use ruma_identifiers::DeviceId;

use crate::r0::uiaa::{AuthData, IncomingAuthData, UiaaResponse};

ruma_api! {
    metadata: {
        description: "Delete specified devices.",
        method: POST,
        path: "/_matrix/client/r0/delete_devices",
        name: "delete_devices",
        rate_limited: false,
        requires_authentication: true,
    }

    request: {
        /// List of devices to delete.
        pub devices: &'a [Box<DeviceId>],

        /// Additional authentication information for the user-interactive authentication API.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub auth: Option<AuthData<'a>>,
    }

    response: {}

    error: UiaaResponse
}
