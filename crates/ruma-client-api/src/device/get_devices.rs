//! `GET /_matrix/client/*/devices`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3devices

    use ruma_common::api::ruma_api;

    use crate::device::Device;

    ruma_api! {
        metadata: {
            description: "Get registered devices for authenticated user.",
            method: GET,
            name: "get_devices",
            r0_path: "/_matrix/client/r0/devices",
            stable_path: "/_matrix/client/v3/devices",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
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
            Self {}
        }
    }

    impl Response {
        /// Creates a new `Response` with the given devices.
        pub fn new(devices: Vec<Device>) -> Self {
            Self { devices }
        }
    }
}
