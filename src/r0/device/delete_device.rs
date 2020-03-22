//! [DELETE /_matrix/client/r0/devices/{deviceId}](https://matrix.org/docs/spec/client_server/r0.6.0#delete-matrix-client-r0-devices-deviceid)

use crate::r0::account::AuthenticationData;
use ruma_api::ruma_api;
use ruma_identifiers::DeviceId;

ruma_api! {
    metadata {
        description: "Delete a device for authenticated user.",
        method: DELETE,
        name: "delete_device",
        path: "/_matrix/client/r0/devices/:device_id",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The device to delete.
        #[ruma_api(path)]
        pub device_id: DeviceId,
        /// Additional authentication information for the user-interactive authentication API.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub auth: Option<AuthenticationData>,
    }

    response {}

    error: crate::Error
}
