//! `POST /_matrix/client/*/delete_devices`
//!
//! Delete specified devices.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3delete_devices

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedDeviceId,
    };

    use crate::uiaa::{AuthData, UiaaResponse};

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/delete_devices",
            1.1 => "/_matrix/client/v3/delete_devices",
        }
    };

    /// Request type for the `delete_devices` endpoint.
    #[request(error = UiaaResponse)]
    pub struct Request {
        /// List of devices to delete.
        pub devices: Vec<OwnedDeviceId>,

        /// Additional authentication information for the user-interactive authentication API.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub auth: Option<AuthData>,
    }

    /// Response type for the `delete_devices` endpoint.
    #[response(error = UiaaResponse)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given device list.
        pub fn new(devices: Vec<OwnedDeviceId>) -> Self {
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
