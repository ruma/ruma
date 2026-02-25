//! `DELETE /_matrix/client/*/devices/{deviceId}`
//!
//! Delete a device for authenticated user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#delete_matrixclientv3devicesdeviceid

    use ruma_common::{
        DeviceId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    use crate::uiaa::{AuthData, UiaaResponse};

    metadata! {
        method: DELETE,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/devices/{device_id}",
            1.1 => "/_matrix/client/v3/devices/{device_id}",
        }
    }

    /// Request type for the `delete_device` endpoint.
    #[request(error = UiaaResponse)]
    pub struct Request {
        /// The device to delete.
        #[ruma_api(path)]
        pub device_id: DeviceId,

        /// Additional authentication information for the user-interactive authentication API.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub auth: Option<AuthData>,
    }

    /// Response type for the `delete_device` endpoint.
    #[response(error = UiaaResponse)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given device ID.
        pub fn new(device_id: DeviceId) -> Self {
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
