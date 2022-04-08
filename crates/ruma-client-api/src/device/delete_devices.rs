//! `POST /_matrix/client/*/delete_devices`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3delete_devices

    use ruma_common::{api::ruma_api, OwnedDeviceId};

    use crate::uiaa::{AuthData, IncomingAuthData, UiaaResponse};

    ruma_api! {
        metadata: {
            description: "Delete specified devices.",
            method: POST,
            r0_path: "/_matrix/client/r0/delete_devices",
            stable_path: "/_matrix/client/v3/delete_devices",
            name: "delete_devices",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// List of devices to delete.
            pub devices: &'a [OwnedDeviceId],

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
        pub fn new(devices: &'a [OwnedDeviceId]) -> Self {
            Self { devices, auth: None }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
