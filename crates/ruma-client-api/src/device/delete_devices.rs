//! `POST /_matrix/client/*/delete_devices`
//!
//! Delete specified devices.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#post_matrixclientv3delete_devices

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedDeviceId,
    };

    use crate::uiaa::{AuthData, IncomingAuthData, UiaaResponse};

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
    pub struct Request<'a> {
        /// List of devices to delete.
        pub devices: &'a [OwnedDeviceId],

        /// Additional authentication information for the user-interactive authentication API.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub auth: Option<AuthData<'a>>,
    }

    /// Response type for the `delete_devices` endpoint.
    #[response(error = UiaaResponse)]
    #[derive(Default)]
    pub struct Response {}

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
