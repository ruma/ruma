//! `GET /_matrix/client/*/devices`
//!
//! Get registered devices for authenticated user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3devices

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    use crate::device::Device;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/devices",
            1.1 => "/_matrix/client/v3/devices",
        }
    };

    /// Request type for the `get_devices` endpoint.
    #[request(error = crate::Error)]
    #[derive(Default)]
    pub struct Request {}

    /// Response type for the `get_devices` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// A list of all registered devices for this user
        pub devices: Vec<Device>,
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
