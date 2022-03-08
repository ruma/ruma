//! `DELETE /_matrix/client/*/devices/{deviceId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#delete_matrixclientv3devicesdeviceid

    use ruma_common::{api::ruma_api, DeviceId};

    use crate::uiaa::{AuthData, IncomingAuthData, UiaaResponse};

    ruma_api! {
        metadata: {
            description: "Delete a device for authenticated user.",
            method: DELETE,
            name: "delete_device",
            r0_path: "/_matrix/client/r0/devices/:device_id",
            stable_path: "/_matrix/client/v3/devices/:device_id",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
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
            Self {}
        }
    }
}
